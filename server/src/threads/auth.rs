use std::time::Instant;
use std::{net::TcpStream, time::Duration};
use std::sync::mpsc;
use std::thread;

use crate::db::UserInfo;
use crate::endpoints::users::login;
use crate::server::TimedStream;
use crate::{endpoints, http_utils, threads::user_threads};
use http_bytes::http;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::user_threads::UserManagerThreadMessage;

//AuthRequest: the basic packet sent from the main thread to the Auth thread
//packet type determines what route to take
//packet always contains the POST request body as a string
//packet always contains the TcpStream output, passing ownership to thread
//TODO: INCLUDE TIMER, FOR LATENCY METRICS, AND SEND RESULTING LATENCY BACK TO HOST(??)
// - may have to find a way to consolidate latency data to a centralized 'metrics' thread/handler??
pub enum AuthRequest {
    Register { jsondata: String, stream: TimedStream },
    Login { jsondata: String, stream: TimedStream }
}

pub enum AuthError {
    BadRequest,
    BadCredentials
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserToken {
    pub id: Uuid,
    pub username: String,
    pub exp: usize
}
impl UserToken{
    pub fn new(user_info: UserInfo, exp: usize) -> UserToken {
        UserToken{
            id: user_info.id,
            username: user_info.username,
            exp: exp
        }
    }
}

//handle_auth_requests(): waits for and handles messages from host thread
//messages are always AuthRequests
pub fn handle_auth_requests(
    thread_sender: mpsc::Sender<AuthRequest>,
    thread_receiver: mpsc::Receiver<AuthRequest>,
    sender_to_user_threads: mpsc::Sender<UserManagerThreadMessage>
) {
    
    //maybe redundant, but initialize communication channel constants
    let (sender, receiver) = (thread_sender, thread_receiver);
    
    println!("auth thread spawned: {:?}", thread::current().id());

    //iterate through host->thread reception channel, yielding if empty
    for req in receiver.iter() {

        //auth thread timer
        let now = Instant::now();

        //once hearing something, check its type
        match req {
            //register: create user in databases if possible
            AuthRequest::Register {
                jsondata,
                mut stream,
            } => {

                //TODO: split this up some, check for success/failure here instead of endpoint

                let register_result = endpoints::users::register(jsondata);

                //why the fuck did i put a match in the function???
                let _ = http_utils::send_response(match register_result {
                    Ok(token) => {
                        sender_to_user_threads.send(UserManagerThreadMessage::Creation { token: token.clone() });
                        let token = format!("{{\"token\":\"{}\"}}", token);
                        let mut res = http_utils::ok_json(http::StatusCode::CREATED, token).unwrap();
                        http_utils::add_header(&mut res, "Location", "/home");
                        res

                    },
                    Err(why) => {
                        match why {
                            AuthError::BadRequest => {
                                http_utils::bad_request().unwrap()
                            },
                            AuthError::BadCredentials => {
                                http_utils::bad_request().unwrap()
                            }
                        }
                    }
                }, &mut stream);
            }

            //login: authenticate user in auth database, send back a token if valid
            AuthRequest::Login {
                jsondata,
                mut stream,
            } => {
                let login_result = endpoints::users::login(jsondata);

                let _ = http_utils::send_response(match login_result {
                    Ok(token) => {
                        sender_to_user_threads.send(UserManagerThreadMessage::Creation { token: token.clone() });
                        let token = format!("{{\"token\":\"{}\"}}", token);
                        let mut res = http_utils::ok_json(http::StatusCode::CREATED, token).unwrap();
                        http_utils::add_header(&mut res, "Location", "/home");
                        res

                    },
                    Err(why) => {
                        match why {
                            AuthError::BadRequest => {
                                http_utils::bad_request().unwrap()
                            },
                            AuthError::BadCredentials => {
                                http_utils::bad_request().unwrap()
                            }
                        }
                    }
                }, &mut stream);
            }
        }

        //time output
        println!("\tauth thread took: {:?}", now.elapsed());
    }
}