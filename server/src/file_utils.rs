use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, File, Metadata};
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use http_bytes::http;

//CLIENT_FILE_PATH: the location of the files that will be sent to client
const CLIENT_FILE_PATH: &str = "../client/static";

//FILE CACHE IMPLEMENTATION:
//instead of loading and reading a file from the file system every single time,
//store the bytes of the file into this cache
//KNOWN LIMITATIONS: 
// - no differentation of different filepaths, only file name
// - no checking for file changes, if file in cache is edited, program requires restart
static FILE_CACHE: LazyLock<Mutex<HashMap<String, Vec<u8>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

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
pub fn get_file(filename: &OsStr) -> Result<Vec<u8>, String> {
    if filename.is_empty() {
        return Err("empty filename!".to_owned());
    }

    let filename = sanitize_filename(filename);

    //build the file path by sticking it to the end of CLIENT_FILE_PATH
    let mut filepath = PathBuf::from(CLIENT_FILE_PATH);
    filepath.push(filename.clone());

    //debug print
    //println!("attempting to get file from: {:?}", filepath);
    if let Some(file) = FILE_CACHE.lock().unwrap().get(&filename){
        return Ok(file.clone());
    }
    //open the file
    match File::open(filepath.as_path()) {
        //if found, return it
        Ok(file) => {
            let mut reader = BufReader::new(file);
            reader.seek(std::io::SeekFrom::Start(0)).unwrap();
            let file: Vec<u8> = reader.bytes().map(Result::unwrap).collect();
            if env::var("DO_CACHING").unwrap_or_default() == "true" {
                FILE_CACHE.lock().unwrap().insert(filename, file.clone());
            }
            Ok(file)
        },
        //otherwise, return an error with the err string
        Err(err) => Err(err.to_string()),
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

