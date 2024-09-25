//used for reading/handling TCP connection
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::sync::mpsc;

use std::{env, path, thread};

//external crates:
//used for parsing HTTP requests into objects
use httparse;
//used for building HTTP responses from parts
use http_bytes::http;

use crate::router::Router;
use crate::db::{self, Database, UserCredentials};
use crate::http_utils;
use crate::auth;

pub struct Server{
    listener: TcpListener,
    router: Router
}
impl Server{

    pub fn new(address: String) -> Server{
        let listener = TcpListener::bind(&address).expect(&format!("listener should have bound to {}", address)[..]);
        let router = Router::new();

        db::AUTH_DB.read().unwrap().create_table(env::var("AUTH_DATABASE_INIT").expect("AUTH_DATABASE_INIT in .env"));
        db::USER_DB.read().unwrap().create_table(env::var("USER_DATABASE_INIT").expect("USER_DATABASE_INIT in .env"));

        

        Server{listener, router}
    }

    //listen(): loops forever through incoming TCP streams and handles them
    pub fn listen(&self) -> Result<(), String>{

        println!("listening on {:?}", self.listener.local_addr().unwrap());

        let (host_sender, thread_receiver) = mpsc::channel::<auth::AuthRequest>();
        let (thread_sender, host_receiver) = mpsc::channel::<auth::AuthRequest>();

        thread::spawn(move || {
            auth::handle_auth_requests(thread_sender, thread_receiver);
        });

        //request counting statistic
        let mut req_count = 0;

        //iterate through incoming TCP connections/requests
        for stream in self.listener.incoming(){

            match stream{
                Ok(mut stream) => {
                    //count req, print
                    req_count += 1;
                    println!("\n\nrequest! #{}", req_count);

                    //handle the request, get a response
                    let mut response = self.handle_connection(&stream)?;

                    //print the response
                    println!("\nresponse:\n{}", http_utils::stringify_response(&response));

                    //write the response to TCP connection stream, as bytes
                    stream.write_all(&*http_utils::serialize_response(&mut response)).unwrap();

                    //"flush" the stream to send it out
                    stream.flush().unwrap();
                },
                Err(why) => {
                    return Err(format!("stream connection failed!:\n{:?}", why));
                }
            }
        }

        Ok(())
    }

    //handle_connection(): reads the given TCP stream and generates a response, using the given Router
    fn handle_connection(&self, mut stream: &TcpStream) -> Result<http::Response<Vec<u8>>, String>{

        //buffer bytes, to store request in
        let mut buffer = [0; 2048];
        
        //read stream into buffer TODO: handle Result
        stream.read(&mut buffer).unwrap();

        //parse request into req (its headers go into req_headers)
        let mut req_headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut req_headers);

        //req_status: whether the request was successfully received entirely, without data loss
        let req_status = req.parse(&buffer).unwrap();

        //check req_status
        match req_status{
            //if complete,
            httparse::Status::Complete(body_index) => {

                let mut body = buffer[body_index..].to_vec();
                body.retain(|x| *x != 0 as u8);

                //grab the request body
                let mut body = Some(String::from_utf8(body).unwrap().to_owned());
                
                //if its empty, ensure its None
                if (body.clone().unwrap()).is_empty(){
                    body = None;
                }

                //print out request for debugging
                println!("\n{}\nbody: {:?}",http_utils::stringify_request(&req), &body.clone().unwrap_or("NONE".to_owned()));

                let mut path_iterator = path::Path::new(req.path.unwrap()).iter();

                //route the request, and return the response
                Ok(self.router.route(&mut path_iterator, http_utils::RequestMethod::parse(req.method.unwrap(), body)))


            },

            //if only partial,
            httparse::Status::Partial => {
                
                todo!("invalid request handler")
            }
        }
        
    }

}