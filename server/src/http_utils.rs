use http_bytes;
use http_bytes::http::{self, status};
use httparse::{Header, Status};
use std::{
    ffi::OsStr,
    io::{BufReader, Read, Write},
    net::TcpStream,
    path::Path,
};

use crate::file_utils;

const REQ_BODY_TRUNCATE_LEN: usize = 128;

pub fn send_response(
    response: &mut http::Response<Vec<u8>>,
    stream: &mut TcpStream,
) -> Result<(), std::io::Error> {
    //print the response
    println!("\nresponse:\n{}", stringify_response(response));

    //write the response to TCP connection stream, as bytes
    stream.write_all(&*serialize_response(response)).unwrap();

    //"flush" the stream to send it out
    stream.flush()
}

//serialize_response(): takes a mutable reference to a response
pub fn serialize_response(response: &mut http::Response<Vec<u8>>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    http_bytes::write_response_header(response, &mut out).expect("serialization to bytes failed");
    out.append(response.body_mut());

    out
}

//stringify_response: takes a response reference and iterate through it, putting it in a string and returning that
pub fn stringify_response(response: &http::Response<Vec<u8>>) -> String {
    let mut out = format!("{:?} {:?}\r\n", response.version(), response.status());
    for (name, value) in response.headers() {
        out = out + &format!("{}: {}\r\n", name.to_string(), value.to_str().unwrap())[..];
    }

    let mut body = response.body().clone();

    let len = body.len();

    body.truncate(REQ_BODY_TRUNCATE_LEN);

    let new_len = body.len();

    let body = String::from_utf8_lossy(body.as_slice());

    out = out
        + "\r\n"
        + &body
        + "... +"
        + &(std::cmp::max(0, len as i32 - new_len as i32).to_string());

    out
}

//like above but for requests
pub fn stringify_request(req: &httparse::Request) -> String {
    let mut out = format!(
        "method: {}\npath: {}\nversion: {}\nheaders:\n",
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

//ok: builds and returns a generic, empty 200 OK response
pub fn ok() -> Result<http::Response<Vec<u8>>, String> {
    Ok(http::Response::builder()
        .status(200)
        .body(String::from("").as_bytes().to_vec())
        .unwrap())
}

pub fn ok_json(body: String) -> Result<http::Response<Vec<u8>>, String> {
    Ok(http::Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Content-Length", body.len())
        .body(body.as_bytes().to_vec())
        .unwrap())
}

//grabs a file and returns it with a proper HTTP response for the file type
pub fn ok_file(filename: &OsStr) -> Result<http::Response<Vec<u8>>, String> {
    let path = Path::new(filename);

    let file = file_utils::get_file(filename)?;

    let metadata = file.metadata().unwrap();

    let file: Vec<u8> = BufReader::new(file).bytes().map(Result::unwrap).collect();

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
        .status(200)
        .header("Content-Type", content_type)
        .header("Content-length", metadata.len())
        .body(file)
        .unwrap())
}

//builds and returns a generic 400 BAD REQUEST http response
pub fn bad_request() -> Result<http::Response<Vec<u8>>, String> {
    let mut res = ok_file(OsStr::new("400.html"))?;
    *res.status_mut() = status::StatusCode::BAD_REQUEST;
    Ok(res)
}

//builds and returns a 404 NOT FOUND http response, with the 404.html webpage
pub fn not_found() -> Result<http::Response<Vec<u8>>, String> {
    let mut res = ok_file(OsStr::new("404.html"))?;
    *res.status_mut() = status::StatusCode::NOT_FOUND;
    Ok(res)
}
