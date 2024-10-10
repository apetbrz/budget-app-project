use std::time::Instant;
use std::{net::TcpStream, time::Duration};
use std::sync::mpsc;
use std::thread;

use crate::{endpoints, http_utils, threads::user_threads};

use super::user_threads::UserManagerThreadMessage;

//AuthRequest: the basic packet sent from the main thread to the Auth thread
//packet type determines what route to take
//packet always contains the POST request body as a string
//packet always contains the TcpStream output, passing ownership to thread
//TODO: INCLUDE TIMER, FOR LATENCY METRICS, AND SEND RESULTING LATENCY BACK TO HOST(??)
// - may have to find a way to consolidate latency data to a centralized 'metrics' thread/handler??
pub enum AuthRequest {
    Register { jsondata: String, stream: TcpStream },
    Login { jsondata: String, stream: TcpStream },
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
                //yippee!
                println!(
                    "handling registration in background thread!!!!!\n{}",
                    jsondata
                );

                //TODO: split this up some, check for success/failure here instead of endpoint
                //send Creation message to user thread manager
                http_utils::send_response(
                    endpoints::users::register(jsondata).unwrap(),
                    &mut stream,
                )
                .unwrap();
            }

            //login: authenticate user in auth database, send back a token if valid
            //TODO: create a personal user thread to handle requests with token
            AuthRequest::Login {
                jsondata,
                mut stream,
            } => {
                //yippee!!
                println!("handling login in background thread!!!!!\n{}", jsondata);
                
                //TODO: split this up like above!!!
                http_utils::send_response(
                    endpoints::users::login(jsondata).unwrap(),
                    &mut stream,
                )
                .unwrap();
            }
        }

        //time output
        println!("auth thread took: {:?}", now.elapsed());
    }
}
