use std::sync::mpsc;

use crate::db::UserInfo;
use crate::metrics;
use crate::server::TimedStream;
use crate::{endpoints, http_utils};
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
    Register { jsondata: String },
    Login { jsondata: String }
}
pub struct AuthMessage {
    pub stream: TimedStream,
    pub request: AuthRequest
}
impl AuthMessage {
    pub fn register(jsondata: String, stream: TimedStream) -> AuthMessage {
        AuthMessage { stream, request: AuthRequest::Register { jsondata } }
    }
    pub fn login(jsondata: String, stream: TimedStream) -> AuthMessage {
        AuthMessage { stream, request: AuthRequest::Login { jsondata } }
    }
}

pub enum AuthError {
    BadRequest,
    BadCredentials,
    AlreadyExists
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
    thread_sender: mpsc::Sender<AuthMessage>,
    thread_receiver: mpsc::Receiver<AuthMessage>,
    sender_to_user_threads: mpsc::Sender<UserManagerThreadMessage>
) {
    
    //maybe redundant, but initialize communication channel constants
    let (sender, receiver) = (thread_sender, thread_receiver);
    
    eprintln!("\t\tauth thread spawned:\t{}", metrics::thread_name_display());

    //iterate through host->thread reception channel, yielding if empty
    for mut msg in receiver.iter() {

        //auth thread timer
        metrics::arrive(msg.stream.id);

        //once hearing something, check its type
        match msg.request {
            //register: create user in databases if possible
            AuthRequest::Register {
                jsondata,
            } => {

                //TODO: split this up some, check for success/failure here instead of endpoint

                let register_result = endpoints::users::register(jsondata);

                //why the fuck did i put a match in the function???
                let _ = http_utils::send_response(match register_result {
                    Ok((id, token)) => {
                        let token_msg = format!("{{\"token\":\"{}\"}}", token);
                        sender_to_user_threads.send(UserManagerThreadMessage::creation(msg.stream.id, id, token));
                        let mut res = http_utils::ok_json(http::StatusCode::CREATED, token_msg).unwrap();
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
                            },
                            AuthError::AlreadyExists => {
                                http_utils::bad_request_msg("Account already exists!".into()).unwrap()
                            }
                        }
                    }
                }, &mut msg.stream);
            }

            //login: authenticate user in auth database, send back a token if valid
            AuthRequest::Login {
                jsondata,
            } => {
                let login_result = endpoints::users::login(jsondata);

                let _ = http_utils::send_response(match login_result {
                    Ok((id, token)) => {
                        let token_msg = format!("{{\"token\":\"{}\"}}", token);
                        sender_to_user_threads.send(UserManagerThreadMessage::creation(msg.stream.id, id, token));
                        let mut res = http_utils::ok_json(http::StatusCode::CREATED, token_msg).unwrap();
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
                            },
                            _ => {
                                http_utils::bad_request().unwrap()
                            }
                        }
                    }
                }, &mut msg.stream);
            }
        }
        
        metrics::end(msg.stream.id);
    }
}