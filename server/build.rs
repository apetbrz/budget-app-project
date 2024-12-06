use std::fs;
use std::path::Path;

fn main() {
    let file_out = Path::new("./.env");
    if let Err(_) = fs::read(file_out) {
        
        let env_file_contents = 
            "SERVER_PORT=\"3000\"\n\
            SECRET=\"REPLACE_ME\"\n\
            DO_CACHING=true";

        fs::write(
            &file_out,
            env_file_contents
        ).unwrap();

        println!("cargo::warning=.env file generated! please create a secret!");

    }
    else {
        println!("cargo::warning=.env file located! compiling...")
    }

    println!("cargo::rerun-if-changed=./.env")
}