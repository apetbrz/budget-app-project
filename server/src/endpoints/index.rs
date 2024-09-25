use std::path;
use std::ffi::OsStr;

use http_bytes::http;
use crate::http_utils;

pub fn index(_ext: &mut path::Iter, _data: Option<String>) -> Result<http::Response<Vec<u8>>, String>{
    http_utils::ok_file(OsStr::new("index.html"))
}

pub fn hello_world(_ext: &mut path::Iter, _data: Option<String>) -> Result<http::Response<Vec<u8>>, String>{
    http_utils::hello_world()
}

pub fn not_found() -> http::Response<Vec<u8>>{
    http_utils::not_found().unwrap()
}

pub fn bad_request() -> http::Response<Vec<u8>>{
    http_utils::bad_request().unwrap()
}