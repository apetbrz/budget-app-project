//used for reading/handling TCP connection
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;

use std::time::{Duration, Instant};
use std::{env, path, thread};

//external crates:
//used for parsing HTTP requests into objects
use httparse::{self, Header};
//http_bytes replacement for http, as http normally doesnt support raw bytes ??
use http_bytes::http::{self, StatusCode};
//pretty text colors for emphasis :)
use colored::Colorize;

use crate::threads::auth::{self, AuthMessage, AuthRequest};
use crate::{db, endpoints, metrics};
use crate::endpoints::Content;
use crate::http_utils;
use crate::router::Router;
use crate::threads::user_threads::{self, UserManagerThreadMessage};

//the limit on http request size (i cant imagine i'd need more than 1kb)
const MAX_REQUEST_BYTES: usize = 4096;

//time interval (in seconds) for the timeout_clock thread (for checking for inactive user threads)
const TIMEOUT_INTERVAL: u64 = 60;

#[derive(Debug)]
pub struct TimedStream {
    stream: TcpStream,
    pub spawntime: Instant,
    pub id: usize
}
impl TimedStream {
    pub fn new(stream: TcpStream) -> TimedStream {
        TimedStream{stream, spawntime: Instant::now(), id: metrics::start() }
    }
    
    pub fn elapsed(&self) -> Duration{
        self.spawntime.elapsed()
    }
}
impl std::io::Write for TimedStream {
    
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
    
}

pub struct Server {
    listener: TcpListener,
    router: Router,
    auth_thread_sender: Option<mpsc::Sender<AuthMessage>>,
    auth_thread_receiver: Option<mpsc::Receiver<AuthMessage>>,
    users_thread_sender: Option<mpsc::Sender<UserManagerThreadMessage>>,
    users_thread_receiver: Option<mpsc::Receiver<UserManagerThreadMessage>>,
}
//TODO: FIND WAY TO REMOVE THE Option FROM THE STRUCT^^^ its annoying
impl Server {
    pub fn new(address: String) -> Server {
        let listener = TcpListener::bind(&address)
            .expect(&format!("listener should have bound to {}", address)[..]);
        let router = Router::new();

        db::USER_DB
            .read()
            .unwrap()
            .create_table(env::var("AUTH_DATABASE_INIT").expect("AUTH_DATABASE_INIT in .env"));
        db::USER_DB
            .read()
            .unwrap()
            .create_table(env::var("USER_DATABASE_INIT").expect("USER_DATABASE_INIT in .env"));

        Server {
            listener,
            router,
            auth_thread_sender: None,
            auth_thread_receiver: None,
            users_thread_sender: None,
            users_thread_receiver: None,
        }
    }

    pub fn send_message_to_auth_thread(&self, msg: AuthMessage) -> Result<(), mpsc::SendError<AuthMessage>> {
        self.auth_thread_sender.as_ref().unwrap().send(msg)
    }

    pub fn send_message_to_user_thread(&self, msg: UserManagerThreadMessage) -> Result<(), mpsc::SendError<UserManagerThreadMessage>> {
        self.users_thread_sender.as_ref().unwrap().send(msg)
    }

    //listen(): loops forever through incoming TCP streams and handles them
    pub fn listen(&mut self) -> Result<(), String> {

        let (host_sender, thread_receiver) = mpsc::channel::<auth::AuthMessage>();
        let (thread_sender, host_receiver) = mpsc::channel::<auth::AuthMessage>();

        self.auth_thread_sender = Some(host_sender);
        self.auth_thread_receiver = Some(host_receiver);

        let (user_host_sender, user_thread_receiver) = mpsc::channel::<user_threads::UserManagerThreadMessage>();
        let (user_thread_sender, user_host_receiver) = mpsc::channel::<user_threads::UserManagerThreadMessage>();
        let timer_thread_sender = user_host_sender.clone();

        self.users_thread_sender = Some(user_host_sender.clone());
        self.users_thread_receiver = Some(user_host_receiver);

        thread::Builder::new().name("user_master".into()).spawn(move || {
            user_threads::handle_user_threads(user_thread_sender, user_thread_receiver);
        }).expect("failed to create user_master thread: OS error");

        thread::Builder::new().name("authenticator".into()).spawn(move || {
            auth::handle_auth_requests(thread_sender, thread_receiver, user_host_sender);
        }).expect("failed to create authenticator thread: OS error");
        
        thread::Builder::new().name("timeout_clock".into()).spawn(move || {
            generate_timeout_checks(timer_thread_sender);
        }).expect("failed to create timeout_clock thread: OS error");


        //request counting statistic
        let mut req_count = 0;
        
        metrics::finish_startup();
        println!("listening on {:?} from thread\t{}", self.listener.local_addr().unwrap(), metrics::thread_name_display());

        //iterate through incoming TCP connections/requests
        for stream in self.listener.incoming() {

            match stream {
                Ok(stream) => {
                    //count req, print
                    println!("\n{}{}\n", "~~~~~~<[ REQUEST! ]>~~~~~~ ".bold().bright_green(), req_count);
                    req_count += 1;

                    let stream = TimedStream::new(stream);
                    let stream_id = stream.id;

                    metrics::arrive(stream_id);

                    //handle the request, get a response
                    let _ = self.handle_connection(stream);

                    metrics::end(stream_id);
                }
                Err(why) => {
                    return Err(format!("stream connection failed!:\n{:?}", why));
                }
            }

        }

        Ok(())
    }

    //handle_connection(): reads the given TCP stream and sends back a response, using the given Router
    fn handle_connection(&self, mut stream: TimedStream) -> Result<(), std::io::Error> {

        //buffer bytes, to store request in
        let mut buffer: [u8; MAX_REQUEST_BYTES] = [0; MAX_REQUEST_BYTES];
        //TODO: check if size is ok? 4kb? how big are requests really?

        //create a buffered reader to read through the stream input
        let mut reader = BufReader::new(&stream.stream);

        //i was having a difficult issue where sometimes request headers and bodies
        //were arriving at different enough times to where stream.read() would
        //finish the headers before the body arrived, leading to missing data
        //i had to fix this by manually parsing the expected content length
        //and fetching that many bytes, after the headers

        //initialize a string to hold headers
        let mut headers = String::new();

        //read lines from tcp stream until end of headers (empty line)
        loop {
            let bytes_read = reader.read_line(&mut headers).unwrap();
            // \r\f = 2 bytes
            if bytes_read < 3 {
                break;
            }
        }

        //take the headers string, split it by line,
        let body_size = match headers
            .split("\n")
            //find the first line where,
            .find(|line| {
                //it contains the content-length header
                line.to_lowercase().starts_with("content-length")
            }) {
            //if the line exists,
            Some(content_length) => {
                //split it just past the ':' (14 chars, plus : and space)
                content_length
                    .split_at(16)
                    //grab the 2nd half (index 1)
                    .1
                    .trim()
                    //and parse it as an integer (or just a zero if something weird is here)
                    .parse::<usize>()
                    .unwrap_or(0)
            }
            //otherwise, assume 0 request body
            None => 0,
        };

        if body_size > MAX_REQUEST_BYTES {
            return http_utils::send_response(http_utils::empty_response(http::StatusCode::PAYLOAD_TOO_LARGE).unwrap(), &mut stream)
        }

        //the body will be stored in a vec of the exact required size
        let mut body_bytes = vec![0; body_size];

        //read into the body buffer
        reader.read_exact(&mut body_bytes).unwrap();

        //convert headers into bytes
        let headers = headers.as_bytes();

        //concatenate the headers and body vecs
        let mut vec_buf = [headers, body_bytes.as_slice()].concat();

        //resize the new vec buffer to fit into the byte array buffer
        vec_buf.resize(MAX_REQUEST_BYTES, 0);

        //copy the vec into the array buffer
        buffer.copy_from_slice(vec_buf.as_slice());

        //dont need this anymore
        drop(vec_buf);

        //i have to do this [u8] shit because httparse ONLY works on arrays, NOT vectors. L

        //parse request into req (its headers go into req_headers)
        let mut req_headers: [httparse::Header; 64] = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut req_headers);

        //req_status: whether the request was successfully received entirely, without data loss
        let Ok(req_status) = req.parse(&buffer) else {
            http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream);
            return Ok(());
        };

        //print the request method and path
        println!(
            "\t<-- {} {} -- body length: {}",
            req.method.unwrap_or("NONE"),
            req.path.unwrap_or("NONE"),
            body_size
        );

        //Option likely not needed tbh
        let body: Option<String>;

        //check the req status, send bad_request if not complete (probably not needed)
        match req_status {
            //if complete request,
            httparse::Status::Complete(body_index) => {
                
                let str = String::from_utf8_lossy(&body_bytes);

                //if it's empty, ensure its None
                if str.is_empty() {
                    body = None;
                    println!("\t\tbody empty\n");
                }
                else{
                    body = Some(str.into());
                    println!("\t\tbody: {:?}\n", body.as_ref().unwrap());
                }
            }
            //if partial request, just crash. i dont think i even need this
            httparse::Status::Partial => {
                println!("PARTIAL REQUEST???????");
                println!("{:?}", String::from_utf8(buffer.to_vec()));
                return http_utils::send_response(
                    http_utils::content_too_large().unwrap(),
                    &mut stream,
                );
            }
        }

        //print out request for debugging
        //println!("\n{}\nbody: {:?}",http_utils::stringify_request(&req), &body.clone().unwrap_or("NONE".to_owned()));

        //create the path iterator
        let mut path_iterator = path::Path::new(req.path.unwrap()).iter();

        //TODO: REFACTOR THIS MESS LMAO

        //route the request
        match self.router.route(&mut path_iterator, req.method.unwrap()) {
            //if Ok, we landed on an endpoint, so handle it accordingly
            Ok(endpoint) => match endpoint {
                //if its a function, run it, and send its response back to the client
                Content::HandlerFunction(func) => {
                    //send a response:
                    return http_utils::send_response(
                        //the response being the output of the given function (TODO: HANDLE ERROR?)
                        func(&mut path_iterator, body).unwrap(),
                        &mut stream,
                    );
                }
                //if it's a registration endpoint, tell the auth thread to handle it
                //pass the req body (json) and TCP stream
                Content::RegisterRequest => match body {
                    //if the body exists, 
                    Some(body) => {
                        //send a message containing it (and the tcp stream) to the auth thread
                        match self.send_message_to_auth_thread(AuthMessage::register(body.clone(), stream)) {
                            //if successful, great!
                            Ok(()) => {
                                Ok(())
                            },
                            //if failed, the auth thread is broken! cant do anything! crash!
                            Err(send_error) => {
                                //IF THIS IS REACHED, OH NO! I LOST THE TCP STREAM
                                //CRY!! PEE MY PANTS!!! I DONT KNOW!!! LET THE CLIENT TIME OUT!
                                println!("AUTH THREAD LOST!!! - {:?}", send_error);
                                panic!("auth thread failure!")
                            }
                        }
                    },
                    //if the body doesnt exist, dont even bother sending it, jsut send a bad_request back
                    None => http_utils::send_response(
                        http_utils::bad_request().unwrap(),
                        &mut stream,
                    ),
                },
                //if it's a login endpoint, tell the auth thread to handle it
                //pass the req body (json) and TCP stream
                Content::LoginRequest => match body {
                    Some(body) => {
                        match self.send_message_to_auth_thread(AuthMessage::login(body.clone(), stream)) {
                            Ok(()) => {
                                Ok(()) 
                            },
                            Err(send_error) => {
                                //IF THIS IS REACHED, OH NO! I LOST THE TCP STREAM
                                //CRY!! PEE MY PANTS!!! I DONT KNOW!!! LET THE CLIENT TIME OUT!
                                println!("AUTH THREAD LOST!!! - {:?}", send_error);
                                panic!("auth thread failure!")
                            }
                        }
                    },
                    None => http_utils::send_response(
                        http_utils::bad_request().unwrap(),
                        &mut stream,
                    ),
                },
                //if it's a logout endpoint, grab the user's token and tell the user thread to handle the logout
                //pass the token and TCP stream
                Content::LogoutRequest => {
                    //get token from Auth header
                    let token = http_utils::find_header_in_request(&req, "Authorization");

                    //if it exists,
                    if let Some(token) = token {
                        //tell user thread to handle logout
                        self.send_message_to_user_thread(UserManagerThreadMessage::shutdown(stream.id, token, stream));
                        Ok(())
                    }
                    //if not,bad request.
                    else{
                        http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream)
                    }
                    //log out
                },
                //if it's a user command endpoint, grab the user's token and tell the user thread to handle the command
                //pass the token, request bodu (json command), and TCP stream
                Content::UserCommand => {

                    //get the token from the 'authorization' header (if not found, send a bad_request res)
                    let token = match http_utils::find_header_in_request(&req, "authorization"){
                        Some(token) => token,
                        None => return http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream)
                    };

                    //if the body exists, send data to user thread
                    match body.as_ref() {
                        Some(body) => {
                            match self.send_message_to_user_thread(UserManagerThreadMessage::user_command(stream.id, token, body.clone(), stream)){
                                Ok(()) => {
                                    //TODO: Something?
                                    Ok(())
                                },
                                //if send fails, the entire user manager thread is gone! program cannot continue
                                Err(send_error) => {
                                    println!("USER HANDLER THREAD LOST!!! - {:?}", send_error);
                                    panic!("user thread failure!")
                                }
                            }
                        },
                        //if no body, no command! bad request.
                        None => http_utils::send_response(
                            http_utils::bad_request().unwrap(),
                            &mut stream,
                        ),
                    }
                }
                
                Content::UserDataRequest => {

                    let token = match http_utils::find_header_in_request(&req, "authorization") {
                        Some(token) => token,
                        None => return http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream)
                    };
                    self.send_message_to_user_thread(UserManagerThreadMessage::user_data_request(stream.id, token, stream));
                    Ok(())
                }
                Content::File(_) => todo!()
            },
            //if no endpoint is found, run the router's not found handler
            Err(handler) => return http_utils::send_response(handler(), &mut stream),
        }

    }
}

//generate_timeout_checks(): creates a looping timer, that sends a TimeoutCheck message
//to the user manager thread every X seconds
fn generate_timeout_checks(channel: mpsc::Sender<user_threads::UserManagerThreadMessage>) {
    eprintln!("\t\ttimeout thread spawned:\t{}", metrics::thread_name_display());
    loop {
        thread::sleep(Duration::from_secs(TIMEOUT_INTERVAL));
        //eprintln!("timeout check:");
        channel.send(user_threads::UserManagerThreadMessage::timeout_check());
    }
}