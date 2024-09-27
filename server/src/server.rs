//used for reading/handling TCP connection
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;

use std::{env, path, thread};

//external crates:
//used for parsing HTTP requests into objects
use httparse;

use crate::auth::{self, AuthRequest};
use crate::db;
use crate::endpoints::Endpoint;
use crate::http_utils;
use crate::router::Router;

pub struct Server {
    listener: TcpListener,
    router: Router,
    auth_thread_sender: Option<mpsc::Sender<AuthRequest>>,
    auth_thread_receiver: Option<mpsc::Receiver<AuthRequest>>,
}
impl Server {
    pub fn new(address: String) -> Server {
        let listener = TcpListener::bind(&address)
            .expect(&format!("listener should have bound to {}", address)[..]);
        let router = Router::new();

        db::AUTH_DB
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
        }
    }

    //listen(): loops forever through incoming TCP streams and handles them
    pub fn listen(&mut self) -> Result<(), String> {
        println!("listening on {:?}", self.listener.local_addr().unwrap());

        let (host_sender, thread_receiver) = mpsc::channel::<auth::AuthRequest>();
        let (thread_sender, host_receiver) = mpsc::channel::<auth::AuthRequest>();

        self.auth_thread_sender = Some(host_sender);
        self.auth_thread_receiver = Some(host_receiver);

        thread::spawn(move || {
            auth::handle_auth_requests(thread_sender, thread_receiver);
        });

        //request counting statistic
        let mut req_count = 0;

        //iterate through incoming TCP connections/requests
        for stream in self.listener.incoming() {
            let now = std::time::Instant::now();

            match stream {
                Ok(stream) => {
                    //count req, print
                    req_count += 1;
                    println!("\n\nrequest! #{}\n", req_count);

                    //handle the request, get a response
                    self.handle_connection(stream).unwrap();
                }
                Err(why) => {
                    return Err(format!("stream connection failed!:\n{:?}", why));
                }
            }

            println!("\trequest took: {:?}", now.elapsed());
        }

        Ok(())
    }

    //handle_connection(): reads the given TCP stream and sends back a response, using the given Router
    fn handle_connection(&self, mut stream: TcpStream) -> Result<(), std::io::Error> {
        //buffer bytes, to store request in
        let mut buffer: [u8; 4096] = [0; 4096];
        //TODO: check if size is ok? 4kb? how big are requests really?

        //create a buffered reader to read through the stream input
        let mut reader = BufReader::new(stream.try_clone().unwrap());

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

        //the body will be stored in a vec of the exact required size
        let mut body = vec![0; body_size];

        //read into the body buffer
        reader.read_exact(&mut body).unwrap();

        println!("{}", String::from_utf8_lossy(&body));

        //convert headers into bytes
        let headers = headers.as_bytes();

        //concatenate the headers and body vecs
        let mut vec_buf = [headers, body.as_slice()].concat();

        //resize the new vec buffer to fit into the byte array buffer
        vec_buf.resize(4096, 0);

        //copy the vec into the array buffer
        buffer.copy_from_slice(vec_buf.as_slice());

        //i have to do this [u8] shit because httparse ONLY works on arrays, NOT vectors. L

        //parse request into req (its headers go into req_headers)
        let mut req_headers: [httparse::Header; 64] = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut req_headers);

        //req_status: whether the request was successfully received entirely, without data loss
        let req_status = req.parse(&buffer).unwrap();
        println!(
            "{} {}",
            req.method.unwrap_or("NONE"),
            req.path.unwrap_or("NONE")
        );

        let mut body: Option<String>;

        //check the req status (probably redundant tbh. TODO: remove)
        match req_status {
            //if complete request,
            httparse::Status::Complete(body_index) => {
                //httparse::Status::Complete contains the index in our byte buffer that points
                //to the start of the body
                //so grab the bit from that index to the position of the first empty byte
                let buffer =
                    buffer[body_index..buffer.iter().position(|&x| x == 0).unwrap()].to_vec();

                //parse the request body into a String
                body = Some(String::from_utf8(buffer).unwrap().to_owned());

                //if it's empty, ensure its None
                if (body.clone().unwrap()).is_empty() {
                    body = None;
                }
                println!("body: {:?}", &body);
            }
            //if partial request, just crash. i dont think i even need this
            httparse::Status::Partial => {
                todo!("handle partial request")
            }
        }

        //print out request for debugging
        //println!("\n{}\nbody: {:?}",http_utils::stringify_request(&req), &body.clone().unwrap_or("NONE".to_owned()));

        //create the path iterator
        let mut path_iterator = path::Path::new(req.path.unwrap()).iter();

        //route the request
        match self.router.route(&mut path_iterator, req.method.unwrap()) {
            //if Ok, we landed on an endpoint, so handle it accordingly
            Ok(endpoint) => match endpoint {
                //if its a function, run it, and send its response back to the client
                //TODO: RENAME TO Content
                Endpoint::FunctionHandler(func) => {
                    return http_utils::send_response(
                        &mut func(&mut path_iterator, body).unwrap(),
                        &mut stream,
                    );
                }
                Endpoint::RegisterRequest => match body {
                    Some(body) => Ok(self
                        .auth_thread_sender
                        .as_ref()
                        .unwrap()
                        .send(AuthRequest::Register {
                            jsondata: body,
                            stream,
                        })
                        .unwrap()),
                    None => http_utils::send_response(
                        &mut http_utils::bad_request().unwrap(),
                        &mut stream,
                    ),
                },
                Endpoint::LoginRequest => match body {
                    Some(body) => Ok(self
                        .auth_thread_sender
                        .as_ref()
                        .unwrap()
                        .send(AuthRequest::Login {
                            jsondata: body,
                            stream,
                        })
                        .unwrap()),
                    None => http_utils::send_response(
                        &mut http_utils::bad_request().unwrap(),
                        &mut stream,
                    ),
                },
            },
            Err(handler) => return http_utils::send_response(&mut handler(), &mut stream),
        }
    }
}
