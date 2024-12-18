use std::sync::{LazyLock, RwLock};

use r2d2_sqlite;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//this static variable IS THE DATABASE
//im statically initializing a RwLock-ed reference
//to a pool of database connections, to the file
//... YES, HAVING ONE STATIC GLOBAL DATABASE REFERENCE IS BAD. TODO: MOVE DATABASE INTO SERVER INSTANCE
pub static USER_DB: LazyLock<RwLock<Database>> =
    LazyLock::new(|| RwLock::new(Database::new("db")));

pub struct Database {
    connection_pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
}
impl Database {
    pub fn new(name: &str) -> Database {
        let _ = std::fs::create_dir("db"); //if err, nothing changes
        let manager = r2d2_sqlite::SqliteConnectionManager::file(format!("db/{}.db", name));
        let pool = r2d2::Pool::new(manager)
            .expect(&format!("error creating r2d2 sqlite pool for {}", name)[..]);

        Database {
            connection_pool: pool,
        }
    }

    pub fn create_table(&self, table: String) {
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
