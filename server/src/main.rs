//dotenv: enables using .env file for variables
//TODO: verbose debug print variable
extern crate dotenv;
use dotenv::dotenv;
use std::env;

//internal modules:
//holds server struct
mod server;
//used for managing/creating HTTP responses/requests
mod http_utils;
//used for interacting with files
mod file_utils;
//used for routing user connections
mod router;
//used for holding endpoint handler functions
mod endpoints;
//used for managing database
mod db;
//used for holding thread code
mod threads;
//used for budgeting functionality
mod budget;

//entrypoint
fn main() -> Result<(), String> {
    
    //set .env variables
    env::set_var("SERVER_PORT", "3000");

    env::set_var("AUTH_DATABASE_NAME", "auth");
    env::set_var("USER_DATABASE_NAME","users");
    
    env::set_var("AUTH_DATABASE_INIT", "auth(uuid TEXT UNIQUE NOT NULL, username TEXT UNIQUE NOT NULL, password TEXT NOT NULL, PRIMARY KEY (uuid))");
    env::set_var("USER_DATABASE_INIT", "users(uuid TEXT UNIQUE NOT NULL, jsondata TEXT NOT NULL, jsonhistory TEXT NOT NULL, PRIMARY KEY (uuid))");
    
    env::set_var("DO_CACHING", "false");// current implementation of "false" isnt great, but works
    env::set_var("MINIMUM_LOGGING_IMPORTANCE", "0");
    
    env::set_var("RUST_BACKTRACE","0");

    //load .env variables
    dotenv().expect("file should load: /server/.env");

    //default host address: localhost:3000
    let host_address = format!(
        "127.0.0.1:{}",
        env::var("SERVER_PORT").expect("SERVER_PORT value in .env file")
    );

    let mut server = server::Server::new(host_address);

    server.listen()
}
