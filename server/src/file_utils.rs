use std::fs::File;
use std::path;
use std::io::prelude::*;

const static_files: Path = Path::new("/../../client/static");

pub fn get_file_as_string(name: String) -> Result<String, String>{
    match File::open(name){
        Ok(mut file) => {
            let mut buf: String = String::new();
            file.read_to_string(&mut buf);
            Ok(buf)
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}