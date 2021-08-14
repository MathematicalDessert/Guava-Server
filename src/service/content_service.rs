use mongodb::{Database, bson::doc};
use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct ContentService {
    db: Database
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(u32)]
pub enum GuavaContentType {
    None = 0,
    Sound = 1,
    Video = 2,
}

#[derive(Clone, Deserialize)]
pub struct Content {
    content_id: String,
    content_type: GuavaContentType,
    hash: String,
}

impl ContentService {
    pub fn new(db: Database) -> Self {
        ContentService {
            db
        }
    }

    pub async fn get_hash_from_id(&self, id: String) -> Result<String, ()> {
        let collection = self.db.collection::<Content>("content");

        match collection.find_one(doc! {
            "content_id": id
        }, None).await {
            Ok(content) => {
                match content {
                    Some(content_unwrapped) => Ok(content_unwrapped.hash),
                    None => Err(()), 
                }
            },
            Err(_) => Err(()),
        }
    }
}