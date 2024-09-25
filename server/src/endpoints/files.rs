use std::path::{self, Path};
use std::ffi::{OsStr, OsString};

use http_bytes::http;
use crate::http_utils;

pub fn get_file(ext: &mut path::Iter, _data: Option<String>) -> Result<http::Response<Vec<u8>>, String>{
    let filename: OsString = ext.collect::<Vec<&OsStr>>().join(OsStr::new("/"));
    println!("attempting to get file named {:?}", filename);
    http_utils::ok_file(filename.as_os_str())
}

pub fn favicon(_ext: &mut path::Iter, _data: Option<String>) -> Result<http::Response<Vec<u8>>, String>{
    let mut filepath = Path::new("favicon.ico").iter();
    get_file(&mut filepath, None)
}