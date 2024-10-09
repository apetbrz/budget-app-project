use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc;

use std::time::{Duration, Instant};
use std::{env, path, thread};

use uuid::Uuid;

use crate::budget::Budget;
use crate::endpoints::users;
use crate::http_utils;

const SECONDS_TO_TIMEOUT_USER_THREAD: u64 = 1800;

pub enum UserManagerThreadMessage {
    Creation {
        token: String,
        id: Uuid
    },
    UserCommand {
        token: String,
        jsondata: String,
        stream: TcpStream
    },
    Shutdown {
        token: String,
        stream: TcpStream,
    },
    TimeoutCheck,
}

enum UserThreadCommand{
    User{
        jsondata: String,
        stream: TcpStream
    },
    Shutdown,
    TimeoutCheck,
    Check
}

//handle_user_threads(): manage all threads for logged-in users
//serves to listen for messages from the main thread and create new user threads
//or pass messages to existing ones.
pub fn handle_user_threads(
    thread_sender_to_main: mpsc::Sender<UserManagerThreadMessage>,
    thread_receiver_from_main: mpsc::Receiver<UserManagerThreadMessage>,
) {
    //create a map to link jsonwebtokens to users
    let mut thread_map: HashMap<String, mpsc::Sender<UserThreadCommand>> = HashMap::new();

    //listen to host
    for msg in thread_receiver_from_main.iter() {
        
        //check message type
        match msg {

            //Creation: create a new thread, linking a JSONWEBTOKEN to a UUID
            UserManagerThreadMessage::Creation { token, id } => {
                
                //create the channel
                let (host_sender, thread_receiver) = mpsc::channel::<UserThreadCommand>();
                
                //insert the thread link into the map
                thread_map.insert(token, host_sender);
                
                //spawn the thread
                thread::spawn(move || {
                    handle_user(id, thread_receiver);
                });
                println!("thread spawned for {}", id);
            },

            //UserCommand: pass a user request to an existing user thread
            UserManagerThreadMessage::UserCommand {
                token,
                jsondata,
                mut stream,
            } => {
                
                //check the thread_map
                match thread_map.get(&token) {
                    
                    //if it exists
                    Some(sender) => {
                        
                        //send the data
                        sender.send(UserThreadCommand::User{jsondata, stream});
                    },
                    //otherwise,
                    None => {
                        //send an unauthorized response (token is invalid)
                        http_utils::send_response(
                            &mut http_utils::unauthorized().unwrap(),
                            &mut stream,
                        )
                        .unwrap();
                    }
                }
            },

            //
            UserManagerThreadMessage::Shutdown { token, mut stream } => {
                match thread_map.get(&token){
                    Some(sender) => {
                        sender.send(UserThreadCommand::Shutdown);
                    },
                    None => {
                        //doesnt exist, do nothing lol
                    }
                }
            }
            UserManagerThreadMessage::TimeoutCheck => {
                for (k, v) in thread_map.iter() {
                    v.send(UserThreadCommand::TimeoutCheck);
                }
                thread::sleep(Duration::from_millis(100));
                thread_map.retain(|k, v| {
                    if let Err(_) = v.send(UserThreadCommand::Check) {
                        false
                    } else {
                        true
                    }
                });
            }
        }
    }
}

fn handle_user(uuid: Uuid, receiver: mpsc::Receiver<UserThreadCommand>) {
    //TODO: INSERT BUDGET APP CREATION HERE, GRABBING USER DATA FROM DATABASE AND STORING IN MEMORY

    //keep track of how long since last command, for timing out
    let mut time_of_last_command = Instant::now();

    //load user data from database TODO: MOVE CALL INTO db.rs INSTEAD OF users.rs
    let mut user: Budget = users::get_user_data_from_uuid(uuid);

    //loop through messages from manager
    'thread_loop: for mut msg in receiver.iter() {
        
        if let UserThreadCommand::Check = msg {
            continue 'thread_loop;
        }

        if let UserThreadCommand::Shutdown = msg {
            break 'thread_loop;
        } 

        if let UserThreadCommand::TimeoutCheck = msg {
            if time_of_last_command.elapsed() > Duration::from_secs(SECONDS_TO_TIMEOUT_USER_THREAD) {
                break 'thread_loop;
            }
        }

        let mut msg = match msg{
            UserThreadCommand::User { jsondata, stream } => {
                time_of_last_command = Instant::now();
                ( jsondata, stream )
            },
            _ => todo!("unimplemented command type!")
        };

        //parse json message
        let json: serde_json::Value = serde_json::from_str(&msg.0).unwrap();

        //initialize json object
        let obj: serde_json::Map<String, serde_json::Value>;

        //if the parsed json is an Object, store it in obj
        if let serde_json::Value::Object(map) = json {
            obj = map;
        }
        //???
        else {
            println!("what? how did i receive a json object that wasnt an Object");
            http_utils::send_response(&mut http_utils::bad_request().unwrap(), &mut msg.1);
            continue 'thread_loop;
            //TODO: do something
        }

        //grab the command out of the object
        let command = obj.get("command");

        //if the command isnt there, its invalid!
        if let None = command {
            http_utils::send_response(&mut http_utils::bad_request().unwrap(), &mut msg.1);
            continue 'thread_loop;
        }

        //if the command is String,
        if let serde_json::Value::String(command) = command.unwrap(){
            
            //match it to get the command to run
            match command.as_str() {
                _ => {
                    http_utils::send_response(&mut http_utils::bad_request().unwrap(), &mut msg.1);
                    //unimplemented
                }

            }//end command match
        }
        else{
            http_utils::send_response(&mut http_utils::bad_request().unwrap(), &mut msg.1);
        }

    }
}
