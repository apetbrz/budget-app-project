use std::collections::HashMap;
use std::sync::mpsc;

use std::time::{Duration, Instant};
use std::thread;

use http_bytes::http::StatusCode;
use uuid::Uuid;

use crate::budget::{self, Budget};
use crate::endpoints::{self, users};
use crate::{http_utils, metrics};
use crate::server::TimedStream;

const SECONDS_TO_TIMEOUT_USER_THREAD: u64 = 30 * 60;

pub struct UserManagerThreadMessage {
    pub id: Option<usize>,
    pub msg: UserManagerMessageType
}
impl UserManagerThreadMessage {
    pub fn creation(id: usize, uuid: Uuid, token: String) -> UserManagerThreadMessage {
        UserManagerThreadMessage { id: Some(id), msg: UserManagerMessageType::Creation { id: uuid, token } }
    }
    pub fn user_command(id: usize, token: String, jsondata: String, stream: TimedStream) -> UserManagerThreadMessage {
        UserManagerThreadMessage { id: Some(id), msg: UserManagerMessageType::UserCommand { token, jsondata, stream } }
    }
    pub fn user_data_request(id: usize, token: String, stream: TimedStream) -> UserManagerThreadMessage {
        UserManagerThreadMessage { id: Some(id), msg: UserManagerMessageType::UserDataRequest { token, stream } }
    }
    pub fn shutdown(id: usize, token: String, stream: TimedStream) -> UserManagerThreadMessage {
        UserManagerThreadMessage { id: Some(id), msg: UserManagerMessageType::Shutdown { token, stream } }
    }
    pub fn timeout_check() -> UserManagerThreadMessage {
        UserManagerThreadMessage { id: None, msg: UserManagerMessageType::TimeoutCheck }
    }
}

pub enum UserManagerMessageType {
    Creation {
        id: Uuid,
        token: String,
    },
    UserCommand {
        token: String,
        jsondata: String,
        stream: TimedStream,
    },
    UserDataRequest {
        token: String,
        stream: TimedStream,
    },
    Shutdown {
        token: String,
        stream: TimedStream,
    },
    TimeoutCheck,
}

struct UserThreadMessage {
    id: Option<usize>,
    cmd: UserThreadCommandType
}
impl UserThreadMessage {
    pub fn user_command(id: Option<usize>, jsondata: String, stream: TimedStream) -> UserThreadMessage {
        UserThreadMessage { id, cmd: UserThreadCommandType::UserCommand { jsondata, stream } }
    }
    pub fn user_data_request(id: Option<usize>, stream: TimedStream) -> UserThreadMessage {
        UserThreadMessage { id, cmd: UserThreadCommandType::UserDataRequest { stream } }
    }
    pub fn shutdown(id: Option<usize>) -> UserThreadMessage {
        UserThreadMessage { id, cmd: UserThreadCommandType::Shutdown }
    }
    pub fn timeout_check() -> UserThreadMessage {
        UserThreadMessage { id: None, cmd: UserThreadCommandType::TimeoutCheck }
    }
    pub fn check(id: Option<usize>) -> UserThreadMessage {
        UserThreadMessage { id, cmd: UserThreadCommandType::Check }
    }
}
enum UserThreadCommandType {
    UserCommand { jsondata: String, stream: TimedStream },
    UserDataRequest { stream: TimedStream },
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
    let mut thread_map: HashMap<String, mpsc::Sender<UserThreadMessage>> = HashMap::new();

    eprintln!("\t\tuser manager thread spawned:\t{}", metrics::thread_name_display());

    //listen to host
    for msg in thread_receiver_from_main.iter() {

        let mut timeout = false;
        if let UserManagerMessageType::TimeoutCheck = msg.msg {timeout = true}
        else if let Some(id) = msg.id { metrics::arrive(id) };

        //check message type
        match msg.msg {
            //Creation: create a new thread, linking a JSONWEBTOKEN to a UUID
            UserManagerMessageType::Creation { id, token } => {
                //create the channel
                let (host_sender, thread_receiver) = mpsc::channel::<UserThreadMessage>();
                let thread_token = token.clone();

                //spawn the thread
                let Ok(handle) = thread::Builder::new().name(id.to_string()).spawn(move || {
                    handle_user(id, thread_token, thread_receiver);
                }) else { 
                    eprintln!("failure to create thread for user {:?} !", id);
                    continue;
                };
                
                //insert the thread link into the map
                thread_map.insert(token, host_sender);

                println!("\t\t{} - currently managing {} threads", metrics::thread_name_display(), thread_map.len());
            }

            //UserCommand: pass a user request to an existing user thread
            UserManagerMessageType::UserCommand {
                token,
                jsondata,
                mut stream,
            } => {
                //check the thread_map
                match thread_map.get(&token) {
                    //if it exists
                    Some(sender) => {
                        //send the data
                        sender.send(UserThreadMessage::user_command(msg.id, jsondata, stream));
                    }
                    //otherwise,
                    None => {
                        //send an unauthorized response (token is invalid)
                        http_utils::send_response(http_utils::unauthorized().unwrap(), &mut stream)
                            .unwrap();
                    }
                }
            }
            //UserDataRequest: return requested loaded user data
            UserManagerMessageType::UserDataRequest { 
                token, 
                mut stream 
            } => {
                match thread_map.get(&token) {
                    Some(sender) => {
                        sender.send(UserThreadMessage::user_data_request(msg.id, stream));
                    }
                    None => {
                        http_utils::send_response(http_utils::unauthorized().unwrap(), &mut stream)
                            .unwrap();
                    }
                }
            }
            //Shurdown: kill thread
            UserManagerMessageType::Shutdown { token, mut stream } => {
                match thread_map.get(&token) {
                    Some(sender) => {
                        sender.send(UserThreadMessage::shutdown(msg.id));
                        thread_map.remove(&token);
                        http_utils::send_response(http_utils::empty_response(StatusCode::OK).unwrap(), &mut stream);
                    }
                    None => {
                        //doesnt exist, do nothing lol
                        http_utils::send_response(http_utils::not_found().unwrap(), &mut stream);
                    }
                }
            }
            //TimeoutCheck: check all threads for timeout
            UserManagerMessageType::TimeoutCheck => {
                let output = format!("  [ user manager timeout check ] : {} -> ", thread_map.len());

                //TODO: wait for response (of "all good!" or "im dead!") instead of looping twice!!!
                for (k, v) in thread_map.iter() {
                    v.send(UserThreadMessage::timeout_check());
                }
                thread::sleep(Duration::from_millis(50));
                thread_map.retain(|k, v| {
                    v.send(UserThreadMessage::check(msg.id)).is_ok()
                });
                println!("{}{} threads after timeout\n", output, thread_map.len());
            }
        }

        if timeout {}
        else if let Some(id) = msg.id { metrics::end(id) };
    }
}

fn handle_user(id: Uuid, token: String, receiver: mpsc::Receiver<UserThreadMessage>) {
    
    println!("\t\t\tuser thread spawned:\t{}", metrics::thread_name_display());

    //keep track of how long since last command, for timing out
    let mut time_of_last_command = Instant::now();

    //load user data from database TODO: MOVE CALL INTO db.rs INSTEAD OF users.rs
    let mut user_budget: Budget = users::get_user_data_from_uuid(id);

    //loop through messages from manager
    'thread_loop: for msg in receiver.iter() {

        if let Some(id) = msg.id { metrics::arrive(id) };

        match msg.cmd {
            //UserDataRequest: json stringify the loaded budget data
            UserThreadCommandType::UserDataRequest { mut stream } => {
                time_of_last_command = Instant::now();
                let jsondata = serde_json::to_string(&user_budget).unwrap();
                http_utils::send_response(http_utils::ok_json(StatusCode::OK, jsondata).unwrap(), &mut stream);
                continue 'thread_loop;
            }
            //Shutdown: exit thread loop 
            UserThreadCommandType::Shutdown => {
                println!("shutting down thread {:?} : {:?}", thread::current().id(), id);
                break 'thread_loop;
            }
            //TimeoutCheck: check how long since last command, and shut down if too long
            UserThreadCommandType::TimeoutCheck => {
                if time_of_last_command.elapsed() > Duration::from_secs(SECONDS_TO_TIMEOUT_USER_THREAD)
                {
                    println!("shutting down thread {:?} : {:?} due to timeout", thread::current().id(), id);
                    break 'thread_loop;
                }
                continue 'thread_loop;
            }
            //Check: do nothing, used for checking that channel still exists
            UserThreadCommandType::Check => continue 'thread_loop,

            //UserCommand: receive a command from the client, act accordingly
            UserThreadCommandType::UserCommand { jsondata, mut stream } => {
                
                time_of_last_command = Instant::now();
                
                //parse json message
                let obj = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&jsondata);

                if let Err(_) = obj {
                    http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream);
                    continue 'thread_loop;
                }

                let obj = obj.unwrap();

                //grab the command out of the object
                let command = obj.get("command");

                //if the command isnt there, its invalid!
                if let None = command {
                    http_utils::send_response(http_utils::bad_request().unwrap(), &mut stream);
                    continue 'thread_loop;
                }

                let command = command.unwrap().as_str().unwrap_or("");
                 
                let result: Result<String, String> = 'command: { match command {
                    "new" => {
                        let Some(label) = obj.get("label") else { break 'command Err("missing_new_label_field".into()) };
                        let Some(label) = label.as_str() else { break 'command Err("invalid_new_label_field".into()) };

                        let Some(amount) = obj.get("amount") else { break 'command Err("missing_new_amount_field".into()) };
                        let Some(amount) = amount.as_str() else { break 'command Err("invalid_new_amount_field".into()) };
                        let Ok(amount) = amount.parse::<f64>() else { break 'command Err("invalid_new_amount_value".into()) };

                        user_budget.add_expense(label, budget::dollars_to_cents(amount));
                    
                        serde_json::to_string(&user_budget).map_err(|_err| "failed_to_build_json".into())
                    }
                    "getpaid" => {
                        match obj.get("amount") {
                            Some(amount) => {
                                let Some(amount) = amount.as_str() else { break 'command Err("invalid_paid_amount_field".into()) };
                                let Ok(amount) = amount.parse::<f64>() else { break 'command Err("invalid_paid_amount_value".into()) };
        
                                user_budget.get_paid_value(budget::dollars_to_cents(amount));
                            },
                            None => {
                                user_budget.get_paid();
                            }
                        }
                    
                        serde_json::to_string(&user_budget).map_err(|_err| "failed_to_build_json".into())
                    }
                    "setincome" => {
                        let Some(amount) = obj.get("amount") else { break 'command Err("missing_income_amount_field".into()) };
                        let Some(amount) = amount.as_str() else { break 'command Err("invalid_income_amount_field".into()) };
                        let Ok(amount) = amount.parse::<f64>() else { break 'command Err("invalid_income_amount_value".into()) };

                        user_budget.set_income(budget::dollars_to_cents(amount));

                        serde_json::to_string(&user_budget).map_err(|_err| "failed_to_build_json".into())
                    }
                    "raiseincome" => {
                        let Some(amount) = obj.get("amount") else { break 'command Err("missing_raise_amount_field".into()) };
                        let Some(amount) = amount.as_str() else { break 'command Err("invalid_raise_amount_field".into()) };
                        let Ok(amount) = amount.parse::<f64>() else { break 'command Err("invalid_raise_amount_value".into()) };

                        user_budget.add_income(budget::dollars_to_cents(amount));

                        serde_json::to_string(&user_budget).map_err(|_err| "failed_to_build_json".into())
                    }
                    "pay" => {
                        let Some(label) = obj.get("label") else { break 'command Err("missing_payment_label_field".into()) };
                        let Some(label) = label.as_str() else { break 'command Err("invalid_payment_label_field".into()) };

                        let payment_result = match obj.get("amount") {
                            Some(amount) => {
                                let Some(amount) = amount.as_str() else { break 'command Err("invalid_payment_amount_field".into()) };
                                let Ok(amount) = amount.parse::<f64>() else { break 'command Err("invalid_payment_amount_value".into()) };
        
                                user_budget.make_dynamic_payment(label, budget::dollars_to_cents(amount))
                            },
                            None => {
                                user_budget.make_static_payment(label)
                            }
                        };

                        if let Err(msg) = payment_result {
                            break 'command Err(msg)
                        }
                    
                        serde_json::to_string(&user_budget).map_err(|_err| "failed_to_build_json".into())
                    }
                    "save" => {
                        let Some(amount) = obj.get("amount") else { break 'command Err("missing_save_amount_field".into()) };
                        let Some(amount) = amount.as_str() else { break 'command Err("invalid_save_amount_field".into()) };
                        
                        let saving_result = match amount.parse::<f64>(){
                            Ok(amount) => {
                                user_budget.save(budget::dollars_to_cents(amount))
                            },
                            Err(_) => {
                                if amount == "all" {
                                    user_budget.save_all()
                                }
                                else {
                                    break 'command Err("invalid_save_amount_value".into())
                                }
                            }
                        };

                        if let Err(msg) = saving_result {
                            break 'command Err(msg)
                        }

                        serde_json::to_string(&user_budget).map_err(|_err| "failed_to_build_json".into())

                    }

                    _ => {
                        Err("invalid-command".into())
                        //unimplemented
                    }
                }};//end command match
                    
                match result {
                    Ok(output) => {
                        http_utils::send_response(http_utils::ok_json(StatusCode::OK, output).unwrap(), &mut stream);
                    }
                    Err(msg) => {
                        eprintln!("thread for user {:?} failed command execution: {:?}", id, msg);
                        http_utils::send_response(http_utils::bad_request_msg(msg).unwrap(), &mut stream);
                    }
                }

                //save
                endpoints::database::save_user_data(id, &user_budget); 
            }
        }

        if let Some(id) = msg.id { metrics::end(id) };

    }

    endpoints::database::save_user_data(id, &user_budget);
}
