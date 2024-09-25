use std::path;

use http_bytes::http;
use uuid;
use bcrypt;

use crate::{db::{self, UserCredentials}, http_utils};

const HASH_COST: u32 = 10;

pub fn register(_ext: &mut path::Iter, data: Option<String>) -> Result<http::Response<Vec<u8>>, String>{
    let auth_db_access = db::AUTH_DB.read().unwrap();

    let conn = auth_db_access.connection();

    drop(auth_db_access);

    let mut user: UserCredentials = match data{
        Some(data) => {
            println!("DATA: [{}]", data);
            match serde_json::from_str(data.trim()){
                Ok(user) => user,
                Err(err) => {
                    println!("failed to parse json text into credentials object\n{}",err.to_string());
                    return http_utils::bad_request()
                }
            }
        },
        None => return http_utils::bad_request()
    };

    user.password = match bcrypt::hash(user.password, HASH_COST){
        Ok(hash) => hash,
        Err(err) => {
            println!("failed password hash\n{}", err.to_string());
            return http_utils::bad_request()
        }
    };

    let id = uuid::Uuid::new_v4();

    loop {
        match conn.execute("INSERT INTO auth(uuid, username, password) VALUES (?, ?, ?)", rusqlite::params![id, user.username, user.password]){
            Ok(_) => break,
            Err(why) => {
                println!("holy hell, the same UUID generated??\n{}", why.to_string());
                return http_utils::bad_request()
            }
        }
    }

    let user_db_access = db::USER_DB.read().unwrap();

    let conn = user_db_access.connection();

    drop(user_db_access);

    conn.execute("INSERT INTO user(uuid, jsondata, jsonhistory) VALUES (?, ?, ?)", rusqlite::params![id, "{}", "{}"]).unwrap();

    println!("user registered!");

    http_utils::ok()
}

pub fn login(ext: &mut path::Iter, _data: Option<String>) -> Result<http::Response<String>, String>{
    todo!("login not impl yet")
}