use std::ffi::{OsStr, OsString};
use std::fs::{self, File, Metadata};
use std::io::prelude::*;
use std::path::PathBuf;

use http_bytes::http;

//CLIENT_FILE_PATH: the location of the files that will be sent to client
const CLIENT_FILE_PATH: &str = "../client/static";

fn sanitize_filename(filename: &OsStr) -> String {
    //i dont want people to dig through my filesystem using "../" to exit folder
    //so clone the filename to be able to edit it:
    let mut filename = String::from(filename.to_str().unwrap()); //TODO: safe

    //and chop off the first 3 characters if theyre "../"
    while filename.starts_with("../") {
        filename = filename.split_off(3);
    }

    filename
}

//get_file: loads given file and returns if found, or error string if not
pub fn get_file(filename: &OsStr) -> Result<File, String> {
    if filename.is_empty() {
        return Err("empty filename!".to_owned());
    }

    //build the file path by sticking it to the end of CLIENT_FILE_PATH
    let mut filepath = PathBuf::from(CLIENT_FILE_PATH);
    filepath.push(sanitize_filename(filename));

    //debug print
    //println!("attempting to get file from: {:?}", filepath);

    //open the file
    match File::open(filepath.as_path()) {
        //if found, return it
        Ok(file) => Ok(file),
        //otherwise, return an error with the err string
        Err(err) => Err(err.to_string()),
    }
}

//open the given file and return it as a String, instead of a file
pub fn get_file_to_string(filename: &OsStr) -> Result<String, String> {
    let file = get_file(filename);

    match file {
        Ok(mut f) => {
            let mut buf = String::new();

            f.read_to_string(&mut buf);

            Ok(buf)
        }
        Err(err) => Err(err),
    }
}

//grab the file's metadata, without opening it
pub fn get_file_metadata(filename: &OsStr) -> Result<Metadata, String> {
    //build the file path by sticking it to the end of CLIENT_FILE_PATH
    let mut filepath = PathBuf::from(CLIENT_FILE_PATH);
    filepath.push(sanitize_filename(filename));

    //debug print
    println!("attempting to get file metadata from: {:?}", filepath);

    //open the file
    match fs::metadata(filepath.as_path()) {
        //if found, return it
        Ok(data) => Ok(data),
        //otherwise, return an error with the err string
        Err(err) => Err(err.to_string()),
    }
}

