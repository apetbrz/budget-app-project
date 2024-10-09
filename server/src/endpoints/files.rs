use std::ffi::{OsStr, OsString};
use std::path::{self, Path};

use crate::http_utils;
use http_bytes::http::{self, StatusCode};

pub fn get_file(
    ext: &mut path::Iter,
    _data: Option<String>,
) -> Result<http::Response<Vec<u8>>, String> {
    let filename: OsString = ext.collect::<Vec<&OsStr>>().join(OsStr::new("/"));
    //println!("attempting to get file named {:?}", filename);
    http_utils::ok_file(StatusCode::OK, filename.as_os_str())
}

pub fn favicon(
    _ext: &mut path::Iter,
    _data: Option<String>,
) -> Result<http::Response<Vec<u8>>, String> {
    let mut filepath = Path::new("favicon.ico").iter();
    get_file(&mut filepath, None)
}
