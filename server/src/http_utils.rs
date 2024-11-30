use colored::Colorize;
use http_bytes;
use http_bytes::http;
use std::{
    ffi::OsStr, io::Write, path::Path, thread
};

use crate::{file_utils, metrics::{self}, server::TimedStream};

const REQ_BODY_TRUNCATE_LEN: usize = 32;
const SHOW_HEADERS: bool = false;
const SHOW_BODY: bool = true;

pub fn send_response(
    mut response: http::Response<Vec<u8>>,
    stream: &mut TimedStream,
) -> Result<(), std::io::Error> {

    
    //print the response
    println!("{}{}\t| {}\n", "--> ".bright_green().bold(), stream.id, stringify_response(&response));
    
    //write the response to TCP connection stream, as bytes
    stream.write_all(&*serialize_response(&mut response)).unwrap();
    
    //"flush" the stream to send it out
    stream.flush()?;
    
    metrics::response_sent(stream.id);

    let binding = thread::current();
    let thread_name = binding.name().map(|x| x.to_owned()).unwrap_or(format!("{:?}",binding.id()));
    
    //eprintln!("  [ response latency: {:?} ] - sent from {:?} thread", stream.elapsed(), thread_name);
    Ok(())
}

//serialize_response(): takes a mutable reference to a response
//turns it into bytes to be sent
pub fn serialize_response(response: &mut http::Response<Vec<u8>>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    http_bytes::write_response_header(response, &mut out).expect("serialization to bytes failed");
    out.append(response.body_mut());

    out
}

//stringify_response: takes a response reference and iterate through it, putting it in a string and returning that
pub fn stringify_response(response: &http::Response<Vec<u8>>) -> String {
    let mut out = format!("{:?} {:?}\n\t\tbody: ", response.version(), response.status());
    
    if SHOW_HEADERS {
        for (name, value) in response.headers() {
            out = out + &format!("{}: {}\r\n", name.to_string(), value.to_str().unwrap())[..];
        }
    }

    if SHOW_BODY {
        let mut body = response.body().clone();

        if body.len() == 0 {
            out = out + "empty";
        }
        else{
            let len = body.len();

            body.truncate(REQ_BODY_TRUNCATE_LEN);

            let new_len = body.len();

            let body = String::from_utf8_lossy(body.as_slice()).replace("\n","\n\t\t");

            out = out
                + &body
                + "... +"
                + &(std::cmp::max(0, len as i32 - new_len as i32).to_string())
                + " bytes";
        }
    }

    out
}

//like above but for requests
pub fn stringify_request(req: &httparse::Request) -> String {
    let mut out = format!(
        "{} {}\nversion: {}\nheaders:\n",
        req.method.unwrap(),
        req.path.unwrap(),
        req.version.unwrap()
    );
    for header in req.headers.iter() {
        out += format!("{:?}\n", header).as_str();
    }
    out
}

//hello_world: builds and returns a mediocre generic Hello World "html" "page"
pub fn hello_world() -> Result<http::Response<Vec<u8>>, String> {
    let body = "<h1>hello pi world!</h1>";
    Ok(http::Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .header("Content-Length", body.len())
        .body(String::from(body).as_bytes().to_vec())
        .unwrap())
}

//ok: builds and returns a generic, empty response
pub fn empty_response(status: http::StatusCode) -> Result<http::Response<Vec<u8>>, String> {
    Ok(http::Response::builder()
        .status(status)
        .body(String::from("").as_bytes().to_vec())
        .unwrap())
}

//ok_json: builds and returns a response with a json object string as the body (does not stringify)
pub fn ok_json(status: http::StatusCode, body: String) -> Result<http::Response<Vec<u8>>, String> {
    Ok(http::Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Content-Length", body.len())
        .body(body.as_bytes().to_vec())
        .unwrap())
}

//grabs a file and returns it with a proper HTTP response for the file type
pub fn ok_file(
    status: http::StatusCode,
    filename: &OsStr,
) -> Result<http::Response<Vec<u8>>, String> {
    let path = Path::new(filename);

    let file = file_utils::get_file(filename)?;

    let content_type = match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("ico") => "image/ico",
        Some("png") => "image/png",
        Some("jpg") => "image/jpg",
        None => {
            todo!("invalid file extension")
        }
        _ => {
            todo!("unknown file extension")
        }
    };

    Ok(http::Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .header("Content-length", file.len())
        .body(file)
        .unwrap())
}

//builds and returns a generic 400 BAD REQUEST http response
pub fn bad_request() -> Result<http::Response<Vec<u8>>, String> {
    ok_file(http::StatusCode::BAD_REQUEST, OsStr::new("400.html"))
}

pub fn content_too_large() -> Result<http::Response<Vec<u8>>, String> {
    empty_response(http::StatusCode::PAYLOAD_TOO_LARGE)
}

pub fn bad_request_msg(msg: String) -> Result<http::Response<Vec<u8>>, String> {
    ok_json(http::StatusCode::BAD_REQUEST, format!("{{\"error\":\"{}\"}}", msg))
}

//builds and returns a 404 NOT FOUND http response, with the 404.html webpage
pub fn not_found() -> Result<http::Response<Vec<u8>>, String> {
    ok_file(http::StatusCode::NOT_FOUND, OsStr::new("404.html"))
}

pub fn unauthorized() -> Result<http::Response<Vec<u8>>, String> {
    empty_response(http::StatusCode::UNAUTHORIZED)
}

pub fn add_header(res: &mut http::Response<Vec<u8>>, key: &'static str, val: &str) {
    res.headers_mut()
        .insert(key, http::HeaderValue::from_str(val).unwrap());
}

//find_header_in_request(): takes a reference to a request and a target key, and returns the value in the headers (if exists)
pub fn find_header_in_request(req: &httparse::Request, key: &str) -> Option<String> {
    let headers = &req.headers;

    let target = headers.iter().filter(|header| header.name.eq_ignore_ascii_case(key)).next();

    match target{
        Some(header) => Some(String::from_utf8_lossy(header.value).to_string()),
        None => None
    }
}
