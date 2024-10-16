use std::{
    env,
    sync::{LazyLock, RwLock},
};

use r2d2_sqlite;
use rusqlite;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//these two static variables ARE THE DATABASES
//im statically initializing a RwLock-ed reference
//to a pool of database connections, to the file
pub static USER_DB: LazyLock<RwLock<Database>> =
    LazyLock::new(|| RwLock::new(Database::new("user")));

pub struct Database {
    connection_pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
}
impl Database {
    pub fn new(name: &str) -> Database {
        let manager = r2d2_sqlite::SqliteConnectionManager::file(format!("db/{}.db", name));
        let pool = r2d2::Pool::new(manager)
            .expect(&format!("error creating r2d2 sqlite pool for {}", name)[..]);

        Database {
            connection_pool: pool,
        }
    }

    pub fn create_table(&self, table: String) {
        println!("linking to table: {}", table.split("(").next().unwrap());

        self.connection_pool
            .get()
            .unwrap()
            .execute(format!("CREATE TABLE IF NOT EXISTS {}", table).as_str(), [])
            .unwrap();
    }

    pub fn connection(&self) -> r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager> {
        self.connection_pool
            .get()
            .expect("failed to get an sqlite connection from pool")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthRow {
    pub uuid: uuid::Uuid,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}
impl UserCredentials {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
}
impl UserInfo {}
