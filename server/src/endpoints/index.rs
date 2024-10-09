use std::ffi::OsStr;
use std::path;

use crate::http_utils;
use http_bytes::http::{self, StatusCode};

pub fn index(
    _ext: &mut path::Iter,
    _data: Option<String>,
) -> Result<http::Response<Vec<u8>>, String> {
    http_utils::ok_file(StatusCode::OK, OsStr::new("index.html"))
}

pub fn home_page(
    _ext: &mut path::Iter,
    _data: Option<String>,
) -> Result<http::Response<Vec<u8>>, String> {
    http_utils::ok_file(StatusCode::OK, OsStr::new("home.html"))
}

pub fn not_found() -> http::Response<Vec<u8>> {
    http_utils::not_found().unwrap()
}

pub fn bad_request() -> http::Response<Vec<u8>> {
    http_utils::bad_request().unwrap()
}

