use std::sync::mpsc;
use std::thread;

use crate::endpoints;

pub enum AuthRequest{
    Register(String),
    Login(String)
}



pub fn handle_auth_requests(sender: mpsc::Sender<AuthRequest>, receiver: mpsc::Receiver<AuthRequest>){
    let (s, r) = (sender, receiver);

    for req in r.iter(){
        match req{
            AuthRequest::Register(json) => {
                
            },
            AuthRequest::Login(json) => {

            }
        }
    }
}