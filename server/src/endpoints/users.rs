use std::{collections::HashSet, env, path::{self, Path}, sync::mpsc, time::{Duration, Instant}};

use bcrypt;
use http_bytes::http::{self, StatusCode};
use jsonwebtoken;
use uuid::{self, Uuid};

use crate::{
    budget::Budget, db::{self, UserAuthRow, UserCredentials, UserInfo}, http_utils, threads::auth::{self, AuthError}
};
use crate::threads::auth::AuthRequest;

const HASH_COST: u32 = 10;

//register() takes user data as a string, parses it,
//hashes the password, and then inserts into databases
pub fn register(data: String) -> Result<String, AuthError> {
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
            return Err(AuthError::BadRequest);
        }
    };

    //attempt to hash the password
    user.password = match bcrypt::hash(user.password, HASH_COST) {
        //if successful, great!
        Ok(hash) => hash,
        //otherwise, idk what couldve happened tbh. just send a 400
        Err(err) => {
            println!("failed password hash\n{}", err.to_string());
            return Err(AuthError::BadRequest);
        }
    };

    //generate a new uuid
    let id = uuid::Uuid::new_v4();

    //TODO: SQL TRANSACTION RATHER THAN TWO SEPARATE EXECUTIONS

    //attempt to insert the user into the auth table
    match conn.execute(
        "INSERT INTO auth(uuid, username, password) VALUES (?, ?, ?)",
        rusqlite::params![id, user.username, user.password],
    ) {
        //if successful, great!
        Ok(_) => {
            //println!("user {} auth registered", user.username)
        },
        //if not, who knows! some SQL error, print it out and send back a 400
        Err(why) => {
            println!("holy hell, register failed?!\n{}", why.to_string());
            return Err(AuthError::BadRequest);
        }
    }

    //repeat above but for user data table
    let user_db_access = db::USER_DB.read().unwrap();
    let conn = user_db_access.connection();

    drop(user_db_access);

    let new_budget = Budget::new(user.username.clone());

    match conn.execute(
        "INSERT INTO users(uuid, jsondata, jsonhistory) VALUES (?, ?, ?)",
        rusqlite::params![id, serde_json::to_string(&new_budget).unwrap(), "{}"],
    ) {
        Ok(_) => {
            //println!("user {} data registered", user.username)
        },
        Err(why) => {
            println!(
                "failed user data table registration for {}\n{}",
                user.username,
                why.to_string()
            );
            //TODO: delete from auth table
            return Err(AuthError::BadRequest);
        }
    }

    println!("user registered!");
    let user_info = UserInfo {
        id: id,
        username: user.username,
    };

    return Ok(create_token(user_info));
}

//login() takes user data as a string, parses it,
//checks the password against the hash in the database,
//and then (if valid) returns a JSONWEBTOKEN
pub fn login(data: String) -> Result<String, AuthError> {

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
            return Err(AuthError::BadRequest);
        }
    };

    //graab the user's Authentication data from the auth table
    let user_row;

    if let Ok(row) = get_user_auth_row(user.username){
        user_row = row;
    }
    else{
        return Err(AuthError::BadCredentials);
    }

    //verify the input password against the stored hash
    match bcrypt::verify(user.password, user_row.password.as_str()) {
        //if method successful,
        Ok(valid) => {
            //check if valid
            if valid {
                //if so, great! grab the user's public info,
                let user_info = UserInfo {
                    id: user_row.uuid,
                    username: user_row.username,
                };
                //and return a generated token!
                return Ok(create_token(user_info));
            } 
            //if not valid, return a bad_credentials
            else {
                return Err(AuthError::BadCredentials);
            }
        }
        //if error in verification, send bad_request
        Err(why) => {
            return Err(AuthError::BadRequest);
        }
    }
}

//create_token_response() takes in UserInfo, generates a jsonwebtoken, and sends a CREATED response
pub fn create_token(user_info: UserInfo) -> String {
    //token expires in an hour
    let exp = chrono::Utc::now() + chrono::Duration::minutes(60);
    //create token data struct
    let token_data = auth::UserToken::new(user_info, exp.timestamp() as usize);

    //encode the data and return it
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &token_data,
        &jsonwebtoken::EncodingKey::from_secret(
            env::var("SECRET")
                .expect("SECRET should be in .env")
                .as_ref(),
        ),
    )
    .unwrap()
}

//TODO: move to db.rs?!
//get_user_auth_row(): takes in a user's username and grabs their Authentication data from the db
pub fn get_user_auth_row(username: String) -> Result<UserAuthRow, rusqlite::Error>{
    
    //grab a connection from the AUTH database's connection pool
    let conn = db::USER_DB.read().unwrap().connection();

    //prepare the SQL statement to find the user's username
    let mut stmt = conn
        .prepare("SELECT * FROM auth WHERE username = ?")
        .unwrap();

    //get the user data out of the auth table
    //query the row with the user's username
    stmt.query_row(rusqlite::params![username], |row| {
            //once on the row, grab all the data out of it
            Ok(UserAuthRow {
                uuid: row.get::<&str, Uuid>("uuid").unwrap(),
                username: row.get::<&str, String>("username").unwrap(),
                password: row.get::<&str, String>("password").unwrap(),
            })
        })
}

//TODO: MOVE TO db.rs?!
//get_user_data_from_uuid(): takes in a user's unique id, and returns their Budget data from the db
pub fn get_user_data_from_uuid(uuid: Uuid) -> Budget {
    let conn = db::USER_DB.read().unwrap().connection();

    let mut stmt = conn.prepare("SELECT * FROM users WHERE uuid = ?").unwrap();

    stmt.query_row(rusqlite::params![uuid], |row| {
        let data: String = row.get("jsondata").unwrap();
        let bud: Budget = serde_json::from_str(data.as_str()).unwrap();
        Ok(bud)
    }).unwrap()
}

//get_uuid_from_token(): takes in a JSONWEBTOKEN and returns the UUID encoded in it
//if the token is valid. returns failure if invalid
pub fn get_uuid_from_token(token: &String) -> Result<Uuid, String> {

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    let secret: String = env::var("SECRET").expect("SECRET should be in .env");

    //attempt to decode token
    let user_info = jsonwebtoken::decode::<auth::UserToken>(
        token, 
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation
        );
    
    //return data if it exists, or error if not
    match user_info{
        Ok(data) => Ok(data.claims.id),
        Err(why) => {
            println!("invalid token!: {:?}", why);
            Err(String::from("INVALID TOKEN"))
        }
    }
}