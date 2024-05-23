
use chrono::prelude::*;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use crate::utils::StringUtils;

use crate::dal::db;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwtorMessage {
    pub channel: i32,
    #[serde(default = "default_timestamp")]
    pub timestamp: DateTime<Utc>,
    pub from: String,
    pub to: String,
    pub message: String,
    
}

fn default_timestamp() -> DateTime<Utc> {
    Utc::now()
}

impl SwtorMessage {

    pub fn as_json_str(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn as_u64_hash(&self) -> u64 {
        self.as_json_str().as_u64_hash()
    }

    pub fn save_messages_to_db(messages: Vec<SwtorMessage>) {

        let conn = db::get_connection();

        const INSERT_PLAYER: &str = 
        "
            INSERT OR IGNORE INTO 
                Characters (character_name)
            SELECT
                ?1
            WHERE NOT EXISTS ( SELECT 1 FROM Characters WHERE character_name = ?1);
        ";

        let mut stmt = conn.prepare(INSERT_PLAYER).unwrap();
        for message in messages.iter() {

            match stmt.execute(&[&message.from]) {
                Ok(_) => {},
                Err(_err) => {}
            }

        }

        const INSERT_MESSAGE: &str = 
        "
            INSERT OR IGNORE INTO 
                ChatLog (chat_hash, character_id, message)
            SELECT
                ?1,
                C.character_id,
                ?2
            FROM
                Characters C
            WHERE
                ?2->>'from' = C.character_name;
        ";
        
        let mut stmt = conn.prepare(INSERT_MESSAGE).unwrap();
        for message in messages.iter() {

            match stmt.execute(params![message.as_u64_hash() as i64, &message.as_json_str()]) {
                Ok(_) => {},
                Err(_err) => {
                    println!("Error inserting message {}", _err);
                }
            }

        }

    }

}