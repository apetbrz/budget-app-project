
use crate::budget::Budget;
use crate::db;
use uuid::Uuid;

pub fn save_user_data(uuid: Uuid, budget: &Budget) -> Result<String, String>{
    let budget = serde_json::to_string(budget).unwrap();

    let conn = db::USER_DB.read().unwrap().connection();

    let mut stmt = conn.prepare("UPDATE users SET jsondata = ? WHERE uuid = ?").unwrap();

    let result = stmt.query_row(rusqlite::params![budget, uuid], |row| {
        Ok(row.get("jsondata")?)
    });

    match result{
        Ok(jsondata) => Ok(jsondata),
        Err(_) => Err(String::from("not found"))
    }
    
}