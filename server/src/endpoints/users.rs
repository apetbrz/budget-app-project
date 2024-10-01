use std::{path, sync::mpsc};

use bcrypt;
use http_bytes::http;
use jsonwebtoken;
use uuid::{self, Uuid};

use crate::{
    auth::AuthRequest,
    db::{self, UserAuthRow, UserCredentials, UserInfo},
    http_utils,
};

const HASH_COST: u32 = 10;

//register() takes user data as a string, parses it,
//hashes the password, and then inserts into databases
pub fn register(data: String) -> Result<http::Response<Vec<u8>>, String> {
    //grab a connection from the AUTH database's connection pool
    let user_db_access = db::USER_DB.read().unwrap();
    let conn = user_db_access.connection();

    //drop access to the static db access, just in case
    drop(user_db_access);

    //attempt to parse the user from the input String
    let mut user: UserCredentials = match serde_json::from_str(data.trim()) {
        //if successful, great!
        Ok(user) => user,
        //if not, print an error message and return a 400 BAD REQUEST
        Err(err) => {
            println!(
                "failed to parse json text into credentials object\n{}",
                err.to_string()
            );
            return http_utils::bad_request();
        }
    };

    //attempt to hash the password
    user.password = match bcrypt::hash(user.password, HASH_COST) {
        //if successful, great!
        Ok(hash) => hash,
        //otherwise, idk what couldve happened tbh. just send a 400
        Err(err) => {
            println!("failed password hash\n{}", err.to_string());
            return http_utils::bad_request();
        }
    };

    println!("register pw hash: {}", user.password);

    //generate a new uuid
    let id = uuid::Uuid::new_v4();

    //attempt to insert the user into the auth table
    match conn.execute(
        "INSERT INTO auth(uuid, username, password) VALUES (?, ?, ?)",
        rusqlite::params![id, user.username, user.password],
    ) {
        //if successful, great!
        Ok(_) => println!("user {} auth registered", user.username),
        //if not, who knows! some SQL error, print it out and send back a 400
        Err(why) => {
            println!("holy hell, the same UUID generated??\n{}", why.to_string());
            return http_utils::bad_request();
        }
    }

    //repeat above but for user data table
    let user_db_access = db::USER_DB.read().unwrap();
    let conn = user_db_access.connection();

    drop(user_db_access);

    match conn.execute(
        "INSERT INTO users(uuid, jsondata, jsonhistory) VALUES (?, ?, ?)",
        rusqlite::params![id, "{}", "{}"],
    ) {
        Ok(_) => println!("user {} data registered", user.username),
        Err(why) => {
            println!(
                "failed user data table registration for {}\n{}",
                user.username,
                why.to_string()
            );
            //TODO: delete from auth table
            return http_utils::bad_request();
        }
    }

    println!("user registered!");

    //TODO: CREATE JSONWEBTOKEN AND SEND, CREATE THREAD TO HANDLE USER

    http_utils::ok()
}

pub fn login(data: String) -> Result<http::Response<Vec<u8>>, String> {
    //grab a connection from the AUTH database's connection pool
    let auth_db_access = db::USER_DB.read().unwrap();
    let conn = auth_db_access.connection();

    //drop access to the static db access, just in case
    drop(auth_db_access);

    //attempt to parse the user from the input String
    let user: UserCredentials = match serde_json::from_str(data.trim()) {
        //if successful, great!
        Ok(user) => user,
        //if not, print an error message and return a 400 BAD REQUEST
        Err(err) => {
            println!(
                "failed to parse json text into credentials object\n{}",
                err.to_string()
            );
            return http_utils::bad_request();
        }
    };

    println!("login pw hash: {}", user.password);

    let mut stmt = conn
        .prepare("SELECT * FROM auth WHERE username = ?")
        .unwrap();

    let user_row = stmt
        .query_row(rusqlite::params![user.username], |row| {
            println!("{:?}", row.get::<&str, Vec<u8>>("uuid"));
            Ok(UserAuthRow {
                uuid: row.get::<&str, Uuid>("uuid").unwrap(),
                username: row.get::<&str, String>("username").unwrap(),
                password: row.get::<&str, String>("password").unwrap(),
            })
        })
        .unwrap();

    //TODO: handle result

    match bcrypt::verify(user.password, user_row.password.as_str()) {
        Ok(valid) => {
            if valid {
                //successful login! return token
                let user_info = UserInfo {
                    id: user_row.uuid.to_string(),
                    username: user_row.username,
                };
                let token = jsonwebtoken::encode(
                    &jsonwebtoken::Header::default(),
                    &user_info,
                    &jsonwebtoken::EncodingKey::from_secret("superdupersecret".as_ref()),
                )
                .unwrap();

                //TODO: INITIALIZE USER HANDLER THREAD HERE

                return http_utils::ok_json(format!("{}\"token\":\"{}\"{}", "{", token, "}"));
            } else {
                return http_utils::bad_request();
            }
        }
        Err(why) => {
            return http_utils::bad_request();
        }
    }
}
