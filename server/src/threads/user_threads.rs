use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc;

use std::time::{Duration, Instant};
use std::{env, path, thread};

use http_bytes::http::StatusCode;
use uuid::Uuid;

use crate::budget::Budget;
use crate::endpoints::{self, users};
use crate::http_utils;

const SECONDS_TO_TIMEOUT_USER_THREAD: u64 = 1800;

pub enum UserManagerThreadMessage {
    Creation {
        token: String,
    },
    UserCommand {
        token: String,
        jsondata: String,
        stream: TcpStream,
    },
    UserDataRequest {
        token: String,
        stream: TcpStream,
    },
    Shutdown {
        token: String,
        stream: TcpStream,
    },
    TimeoutCheck,
}

#[derive(Debug)]
enum UserThreadCommand {
    UserCommand { jsondata: String, stream: TcpStream },
    UserDataRequest { stream: TcpStream },
    Shutdown,
    TimeoutCheck,
    Check,
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

    println!("user manager thread spawned: {:?}", thread::current().id());

    //listen to host
    for msg in thread_receiver_from_main.iter() {
        
        let now = Instant::now();

        //check message type
        match msg {
            //Creation: create a new thread, linking a JSONWEBTOKEN to a UUID
            UserManagerThreadMessage::Creation { token } => {
                //create the channel
                let (host_sender, thread_receiver) = mpsc::channel::<UserThreadCommand>();

                //insert the thread link into the map
                thread_map.insert(token.clone(), host_sender);

                //spawn the thread
                let handle = thread::spawn(move || {
                    handle_user(token, thread_receiver);
                });

                println!("thread spawned!: {:?}", handle.thread().id());
            }

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
                        sender.send(UserThreadCommand::UserCommand { jsondata, stream });
                    }
                    //otherwise,
                    None => {
                        //send an unauthorized response (token is invalid)
                        http_utils::send_response(http_utils::unauthorized().unwrap(), &mut stream)
                            .unwrap();
                    }
                }
            }

            UserManagerThreadMessage::UserDataRequest { 
                token, 
                mut stream 
            } => {
                match thread_map.get(&token) {
                    Some(sender) => {
                        sender.send(UserThreadCommand::UserDataRequest { stream });
                    }
                    None => {
                        http_utils::send_response(http_utils::unauthorized().unwrap(), &mut stream)
                            .unwrap();
                    }
                }
            }

            
            UserManagerThreadMessage::Shutdown { token, mut stream } => {
                match thread_map.get(&token) {
                    Some(sender) => {
                        sender.send(UserThreadCommand::Shutdown);
                    }
                    None => {
                        //doesnt exist, do nothing lol
                    }
                }
            }
            UserManagerThreadMessage::TimeoutCheck => {
                //TODO: wait for response (of "all good!" or "im dead!") instead of looping twice!!!
                for (k, v) in thread_map.iter() {
                    v.send(UserThreadCommand::TimeoutCheck);
                }
                thread::sleep(Duration::from_millis(10));
                thread_map.retain(|k, v| {
                    if let Err(_) = v.send(UserThreadCommand::Check) {
                        false
                    } else {
                        true
                    }
                });
            }
        }

        println!("\tmaster user thread took: {:?}", now.elapsed())
    }
}

fn handle_user(token: String, receiver: mpsc::Receiver<UserThreadCommand>) {
    //TODO: INSERT BUDGET APP CREATION HERE, GRABBING USER DATA FROM DATABASE AND STORING IN MEMORY
    let id = users::get_uuid_from_token(&token).unwrap();
    
    println!("hello world! from thread {:?} for {:?}", thread::current().id(), id);

    //keep track of how long since last command, for timing out
    let mut time_of_last_command = Instant::now();

    //load user data from database TODO: MOVE CALL INTO db.rs INSTEAD OF users.rs
    let mut user_budget: Budget = users::get_user_data_from_uuid(id);

    //loop through messages from manager
    'thread_loop: for mut msg in receiver.iter() {

        let now = Instant::now();

        if let UserThreadCommand::Check = msg {
            continue 'thread_loop;
        }

        if let UserThreadCommand::Shutdown = msg {
            break 'thread_loop;
        }

        if let UserThreadCommand::TimeoutCheck = msg {
            if time_of_last_command.elapsed() > Duration::from_secs(SECONDS_TO_TIMEOUT_USER_THREAD)
            {
                break 'thread_loop;
            }
            continue 'thread_loop;
        }
        
        if let UserThreadCommand::UserDataRequest { mut stream } = msg {
            let jsondata = serde_json::to_string(&user_budget).unwrap();
            http_utils::send_response(http_utils::ok_json(StatusCode::OK, jsondata).unwrap(), &mut stream);
            continue 'thread_loop;
        }

        let (mut jsondata, mut stream) = match msg {
            UserThreadCommand::UserCommand { jsondata, mut stream } => {
                time_of_last_command = Instant::now();
                (jsondata, stream)
            }
            _ => todo!("unimplemented command type! {:?}", msg),
        };

        //parse json message
        let json: serde_json::Value = serde_json::from_str(&jsondata).unwrap();

        //initialize json object
        let obj: serde_json::Map<String, serde_json::Value>;

        //if the parsed json is an Object, store it in obj
        if let serde_json::Value::Object(map) = json {
            obj = map;
        }
        //???
        else {
            println!("what? how did i receive a json object that wasnt an Object");
            http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream);
            continue 'thread_loop;
            //TODO: do something
        }

        //grab the command out of the object
        let command = obj.get("command");

        //if the command isnt there, its invalid!
        if let None = command {
            http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream);
            continue 'thread_loop;
        }

        //if the command is String,
        if let serde_json::Value::String(command) = command.unwrap() {
            //match it to get the command to run
            match command.as_str() {
                "new" => {
                    
                    let result: Result<String, String> = {
                        let label: String = String::from(obj.get("label").unwrap().as_str().unwrap());
                        let amount = obj.get("amount").unwrap().as_str().unwrap().parse().unwrap();
                        user_budget.add_expense(&label, crate::budget::dollars_to_cents(amount));
                        
                        serde_json::to_string(&user_budget).map_err(|_err| String::from("failed to build json string"))
                    };

                    match result {
                        Ok(output) => {
                            http_utils::send_response(http_utils::ok_json(StatusCode::OK, output).unwrap(), &mut stream);
                        }
                        Err(msg) => {
                            http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream);
                        }
                    }
                }
                "getpaid" => {
                    match obj.get("amount"){
                        Some(value) => {
                            let value: f32 = value.as_str().unwrap().parse().unwrap();
                            user_budget.get_paid_value(crate::budget::dollars_to_cents(value));
                            let output = serde_json::to_string(&user_budget).unwrap();
                            http_utils::send_response(http_utils::ok_json(StatusCode::OK, output).unwrap(), &mut stream);
                        },
                        None => {
                            user_budget.get_paid();
                            let output = serde_json::to_string(&user_budget).unwrap();

                            http_utils::send_response(http_utils::ok_json(StatusCode::OK, output).unwrap(), &mut stream);
                        }
                    }
                }
                "setincome" => {
                    match obj.get("amount"){
                        Some(value) => {
                            let value: f32 = value.as_str().unwrap().parse().unwrap();
                            user_budget.set_income(crate::budget::dollars_to_cents(value));
                            let output = serde_json::to_string(&user_budget).unwrap();
                            http_utils::send_response(http_utils::ok_json(StatusCode::OK, output).unwrap(), &mut stream);
                        },
                        None => {
                            http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream);
                        }
                    }
                }
                _ => {
                    http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream);
                    //unimplemented
                }
            } //end command match
        } else {
            http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream);
        }

        println!("\tuser thread took: {:?} --- user: {:?}", now.elapsed(), id);

    }
}
