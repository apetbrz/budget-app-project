use http_bytes::http::{self, status};
use std::{ffi::OsStr, io::{BufReader, BufWriter, Read, Write}, path::Path};
use serde::{Serialize};
use http_bytes;

use crate::file_utils;

const REQ_BODY_TRUNCATE_LEN: usize = 128;

//RequestMethod: defines supported HTTP req methods, and holds the request body if needed
pub enum RequestMethod{
    GET,
    POST(Option<String>),
    INVALID
}
impl RequestMethod{
    //parse(): takes a string representation of the method type, and optional data, and returns the proper enum
    pub fn parse(str: &str, body: Option<String>) -> RequestMethod{
        let str = str.to_lowercase();
        match str.as_str(){
            "get" => RequestMethod::GET,
            "post" => RequestMethod::POST(body),
            _ => RequestMethod::INVALID
        }
    }
}

//serialize_response(): takes a mutable reference to a response
pub fn serialize_response(response: &mut http::Response<Vec<u8>>) -> Vec<u8>{
    let mut out: Vec<u8> = Vec::new();
    http_bytes::write_response_header(response, &mut out).expect("serialization to bytes failed");
    out.append(response.body_mut());

    out
}

//stringify_response: takes a response reference and iterate through it, putting it in a string and returning that
pub fn stringify_response(response: &http::Response<Vec<u8>>) -> String{
    let mut out = format!("{:?} {:?}\r\n", response.version(), response.status());
    for (name, value) in response.headers(){
        out = out + &format!("{}: {}\r\n",name.to_string(), value.to_str().unwrap())[..];
    }
    
    let mut body = String::from_utf8(response.body().clone()).expect("failed utf8 parse");
    let len = body.len();
    body.truncate(REQ_BODY_TRUNCATE_LEN);

    out = out + "\r\n" + &body + "... +" + &(std::cmp::max(0, len as i32 - REQ_BODY_TRUNCATE_LEN as i32).to_string());

    out
}

//like above but for requests
pub fn stringify_request(req: &httparse::Request) -> String{
    let mut out = format!("method: {}\npath: {}\nversion: {}\nheaders:\n",req.method.unwrap(), req.path.unwrap(), req.version.unwrap());
    for header in req.headers.iter(){
        out += format!("{:?}\n", header).as_str();
    }
    out
}

//hello_world: builds and returns a mediocre generic Hello World "html" "page"
pub fn hello_world() -> Result<http::Response<Vec<u8>>, String>{
    let body = "<h1>hello pi world!</h1>";
    Ok(http::Response::builder()
        .status(200)
        .header("Content-Type","text/html")
        .header("Content-Length", body.len())
        .body(String::from(body).as_bytes().to_vec())
        .unwrap())
}

//ok: builds and returns a generic, empty 200 OK response
pub fn ok() -> Result<http::Response<Vec<u8>>, String>{
    Ok(http::Response::builder()
        .status(200)
        .body(String::from("").as_bytes().to_vec())
        .unwrap())
}

//grabs a file and returns it with a proper HTTP response for the file type
pub fn ok_file(filename: &OsStr) -> Result<http::Response<Vec<u8>>, String>{
    let path = Path::new(filename);

    let file = file_utils::get_file(filename)?;
    
    let metadata = file.metadata().unwrap();

    let file: Vec<u8> = BufReader::new(file).bytes().map(Result::unwrap).collect();

    let content_type = match path.extension().and_then(std::ffi::OsStr::to_str){
        Some("html") => {
            "text/html; charset=utf-8"
        },
        Some("css") => {
            "text/css"
        },
        Some("js") => {
            "text/javascript"
        },
        Some("ico") => {
            "image/ico"
        },
        Some("png") => {
            "image/png"
        },
        Some("jpg") => {
            "image/jpg"
        },
        None => {
            todo!("invalid file extension")
        },
        _ => {
            todo!("unknown file extension")
        }
    };

    Ok(http::Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Content-length",metadata.len())
        .body(file)
        .unwrap())
}

//builds and returns a generic 400 BAD REQUEST http response
pub fn bad_request() -> Result<http::Response<Vec<u8>>, String>{
    let mut res = ok_file(OsStr::new("400.html"))?;
    *res.status_mut() = status::StatusCode::BAD_REQUEST;
    Ok(res)
}

//builds and returns a 404 NOT FOUND http response, with the 404.html webpage
pub fn not_found() -> Result<http::Response<Vec<u8>>, String>{
    let mut res = ok_file(OsStr::new("404.html"))?;
    *res.status_mut() = status::StatusCode::NOT_FOUND;
    Ok(res)
}
