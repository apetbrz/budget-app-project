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
//used for logging and displaying metrics
mod metrics;

//entrypoint
fn main() -> Result<(), String> {

    metrics::begin_startup();
    
    //load .env variables
    dotenv().expect("file should load: /server/.env");

    //default host address: localhost:3000
    let host_address = format!(
        "127.0.0.1:{}",
        env::var("SERVER_PORT").expect("SERVER_PORT value in .env file")
    );

    if env::var("SECRET").expect("SECRET string in .env required!").is_empty() {
        panic!("SECRET string in .env required!")
    };

    let mut server = server::Server::new(host_address);

    server.listen()
}