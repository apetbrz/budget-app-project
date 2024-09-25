//used for reading/handling TCP connection
use std::net::{TcpListener, TcpStream};
use std::io::{prelude::*, BufReader, BufWriter};
use std::sync::mpsc;

use std::{env, path, thread};

//external crates:
//used for parsing HTTP requests into objects
use httparse;

use crate::endpoints::Endpoint;
use crate::router::Router;
use crate::db;
use crate::http_utils;
use crate::auth::{self, AuthRequest};

pub struct Server{
    listener: TcpListener,
    router: Router,
    auth_thread_sender: Option<mpsc::Sender<AuthRequest>>,
    auth_thread_receiver: Option<mpsc::Receiver<AuthRequest>>
}
impl Server{

    pub fn new(address: String) -> Server{
        let listener = TcpListener::bind(&address).expect(&format!("listener should have bound to {}", address)[..]);
        let router = Router::new();

        db::AUTH_DB.read().unwrap().create_table(env::var("AUTH_DATABASE_INIT").expect("AUTH_DATABASE_INIT in .env"));
        db::USER_DB.read().unwrap().create_table(env::var("USER_DATABASE_INIT").expect("USER_DATABASE_INIT in .env"));

        Server{listener, router, auth_thread_sender: None, auth_thread_receiver: None}
    }

    //listen(): loops forever through incoming TCP streams and handles them
    pub fn listen(&mut self) -> Result<(), String>{

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
        for stream in self.listener.incoming(){

            match stream{
                Ok(stream) => {
                    //count req, print
                    req_count += 1;
                    println!("\n\nrequest! #{}\n", req_count);

                    //handle the request, get a response
                    self.handle_connection(stream).unwrap();
                },
                Err(why) => {
                    return Err(format!("stream connection failed!:\n{:?}", why));
                }
            }
        }

        Ok(())
    }

    //handle_connection(): reads the given TCP stream and sends back a response, using the given Router
    fn handle_connection(&self, mut stream: TcpStream) -> Result<(), std::io::Error>{

        //buffer bytes, to store request in
        let mut buffer: [u8; 4096] = [0; 4096];
        
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut headers = String::new();

        loop{
            let bytes_read = reader.read_line(&mut headers).unwrap();
            if bytes_read < 3 { break }
        }

        println!("{}", headers);

        let mut body_size: usize = 0;

        headers.split("\n").for_each(|line| {
            if line.to_lowercase().starts_with("content-length"){
                body_size = line.split_at(16).1.trim().parse::<usize>().unwrap();
            }
        });

        let mut body = vec![0; body_size];

        reader.read_exact(&mut body).unwrap();

        println!("{}", String::from_utf8_lossy(&body));

        let headers = headers.as_bytes();

        let mut vec_buf = [headers, body.as_slice()].concat();

        vec_buf.resize(4096, 0);

        buffer.copy_from_slice(vec_buf.as_slice());

        //parse request into req (its headers go into req_headers)
        let mut req_headers: [httparse::Header; 64] = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut req_headers);

        //req_status: whether the request was successfully received entirely, without data loss
        let req_status = req.parse(&buffer).unwrap();

        let mut body: Option<String>;

        match req_status{
            httparse::Status::Complete(body_index) => {
                let mut buffer = buffer[body_index..].to_vec();

                //TODO: WHY BODY EMPTY SOMETIMES???????
                buffer.retain(|x| *x != 0 as u8);

                //grab the request body
                body = Some(String::from_utf8(buffer).unwrap().to_owned());
                
                //if its empty, ensure its None
                if (body.clone().unwrap()).is_empty(){
                    body = None;
                }

            },
            httparse::Status::Partial => {
                todo!("handle partial request")
            }

        }

        //print out request for debugging
        //println!("\n{}\nbody: {:?}",http_utils::stringify_request(&req), &body.clone().unwrap_or("NONE".to_owned()));

        //create the path iterator
        let mut path_iterator = path::Path::new(req.path.unwrap()).iter();

        //route the request
        match self.router.route(&mut path_iterator, req.method.unwrap()){

            Ok(endpoint) => {
                match endpoint{
                    Endpoint::FunctionHandler(func) => {
                        return http_utils::send_response(&mut func(&mut path_iterator, body).unwrap(), &mut stream);
                    },
                    Endpoint::RegisterRequest => {
                        match body{
                            Some(body) => {
                                Ok(self.auth_thread_sender.as_ref().unwrap().send(AuthRequest::Register(body, stream)).unwrap())
                            },
                            None => {
                                http_utils::send_response(&mut http_utils::bad_request().unwrap(), &mut stream)
                            }
                        }
                        
                    }
                    Endpoint::LoginRequest => {
                        match body{
                            Some(body) => {
                                Ok(self.auth_thread_sender.as_ref().unwrap().send(AuthRequest::Login(body, stream)).unwrap())
                            },
                            None => {
                                http_utils::send_response(&mut http_utils::bad_request().unwrap(), &mut stream)
                            }
                        }
                    }
                }
            },
            Err(handler) => {
                return http_utils::send_response(&mut handler(), &mut stream)
            }

        }
        
    }

}