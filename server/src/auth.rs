use std::sync::mpsc;
use std::thread;
use std::net::TcpStream;

use crate::{endpoints, http_utils};

pub enum AuthRequest{
    Register(String, TcpStream),
    Login(String, TcpStream)
}

pub fn handle_auth_requests(thread_sender: mpsc::Sender<AuthRequest>, thread_receiver: mpsc::Receiver<AuthRequest>){
    let (s, r) = (thread_sender, thread_receiver);

    for req in r.iter(){
        
        match req{
            AuthRequest::Register(json, mut stream) => {
                println!("handling registration in background thread!!!!!\n{}", json);
                http_utils::send_response(&mut endpoints::users::register(json).unwrap(), &mut stream).unwrap();
            },
            AuthRequest::Login(json, stream) => {

            }
        }
    }
}