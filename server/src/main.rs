extern crate dotenv;
use dotenv::dotenv;
use std::env;

use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use httparse;
use http;

mod http_utils;
mod file_utils;

fn main() {
    dotenv().ok();

    let host_address = format!("127.0.0.1:{}", env::var("SERVER_PORT").expect("SERVER_PORT value in .env file"));
    let listener = TcpListener::bind(&host_address).expect(&format!("listener should have bound to {}", host_address)[..]);

    for stream in listener.incoming(){
        let mut stream = stream.unwrap();

        let response = handle_connection(&stream);

        println!("{:?}", response);

        stream.write(http_utils::stringify_response(&response).as_bytes()).unwrap();

        stream.flush().unwrap();
    }
}

fn handle_connection(mut stream: &TcpStream) -> http::Response<String>{

    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let mut req_headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut req_headers);
    let req_status = req.parse(&buffer).unwrap();

    println!("{}",http_utils::stringify_request(&req));

    let body = "<h1>hello pi world!</h1>";

    let res = http::Response::builder()
        .status(200)
        .header("Content-Type","text/html")
        .header("Content-Length", body.len())
        .body(String::from(body))
        .unwrap();

    res

}