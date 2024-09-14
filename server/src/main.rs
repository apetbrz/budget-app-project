extern crate dotenv;
use dotenv::dotenv;
use std::env;

use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use httparse;
use http;

fn main() {
    dotenv().ok();

    let host_address = format!("127.0.0.1:{}", env::var("SERVER_PORT").expect("SERVER_PORT value in .env file"));
    let listener = TcpListener::bind(&host_address).expect(&format!("listener should have bound to {}", host_address)[..]);

    for stream in listener.incoming(){
        let mut stream = stream.unwrap();

        let response = handle_connection(&stream);

        println!("{:?}", response);

        stream.write(response.as_bytes()).unwrap();

        stream.flush().unwrap();
    }

}

fn handle_connection(mut stream: &TcpStream) -> String{
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let mut req_headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut req_headers);
    let req_status = req.parse(&buffer).unwrap();

    debug_print_req(&req);

    let body = "<h1>hello pi world!</h1>";
    let res = http::Response::builder()
        .status(200)
        .header("Content-Type","text/html")
        .header("Content-Length", body.len())
        .body(body.to_owned())
        .unwrap();

    stringify(&res)

}

fn stringify(response: &http::Response<String>) -> String{
    let mut out = format!("{:?} {:?}\r\n", response.version(), response.status());
    for (name, value) in response.headers(){
        out = out + &format!("{}: {}\r\n",name.to_string(), value.to_str().unwrap())[..];
    }
    out = out + "\r\n" + response.body();

    out
}

fn debug_print_req(req: &httparse::Request){
    println!("\n\nmethod: {}\npath: {}\nversion: {}\nheaders:\n",req.method.unwrap(), req.path.unwrap(), req.version.unwrap());
    for header in req.headers.iter(){
        println!("{:?}", header);
    }
}