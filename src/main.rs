use std::env;
use serde_json::{self, Map, Value};
use tide::{Request, Response, StatusCode, prelude::*};
use lazy_static::lazy_static;
use futures::{stream::TryStreamExt};
use mongodb::{Client, Collection, bson::doc, options::{ClientOptions, FindOptions, FindOneOptions}};

lazy_static! {
    static ref MONGO_HOST: String = env::var("MONGO_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    static ref MONGO_PORT: String = env::var("MONGO_PORT").unwrap_or_else(|_| "27017".to_string());
    static ref CONNECTION_STRING: String = format!("mongodb://{}:{}/", MONGO_HOST.as_str(), MONGO_PORT.as_str());
}

#[derive(Clone)]
struct State {
    db: mongodb::Database,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(u32)]
enum GuavaContentType {
    None = 0,
    Sound = 1,
    Video = 2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GuavaContent {
    content_type: GuavaContentType,
    name: String,
    hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GuavaPlaylist {
    name: String,
    identifier: String,
    content: Option<Vec<GuavaContent>>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GuavaPlaylistLight {
    name: String,
    identifier: String,
}

async fn generate_response(status_code: StatusCode, result: Option<Value>, error: Option<String>) -> Response {
    let mut map = Map::new();
    map.insert(String::from("success"), Value::Bool(status_code.is_success()));
    

    if status_code.is_success() {
        map.insert(String::from("result"), result.unwrap_or(serde_json::json!({})));
    } else {
        map.insert(String::from("error"), Value::String(error.unwrap_or("Internal Server Error".to_string())));
    }

    Response::builder(status_code)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&Value::Object(map)).unwrap())
        .build()
}

#[derive(Serialize, Deserialize)]
struct ListPlaylistResponse {
    name: String,
    identifier: String
}

/// List playlist
/// 
/// Lists names of all known playlists
async fn list_playlist(req: Request<State>) -> tide::Result {
    let db = &req.state().db;
    let playlist_collection: Collection<GuavaPlaylistLight> = db.collection("playlist"); 

    let find_options = FindOptions::builder()
        .projection(doc! [
            "content": false,
        ]).build();

    let results = playlist_collection.find(None, find_options).await?;
    let playlists = results.try_collect().await.unwrap_or_else(|_| vec![]);
    
    Ok(generate_response(StatusCode::Ok, Some(serde_json::value::to_value(playlists).unwrap()), None).await)
}

/// Get playlist
/// 
/// Returns list of content under a given playlist
async fn get_playlist_content(req: Request<State>) -> tide::Result {
    let db = &req.state().db;
    let playlist_collection: Collection<GuavaPlaylist> = db.collection("playlist"); 

    match req.param("name") {
        Ok(playlist_name) => {
            // TODO: project results to ignore name + identifier.

            let res = playlist_collection.find_one(doc! { "identifier": playlist_name }, None).await;
            if res.is_err() {
                Ok(generate_response(StatusCode::InternalServerError, None::<Value>, None).await)
            } else {
                match res.ok() {
                    Some(playlist) => {
                        Ok(generate_response(StatusCode::Ok, Some(serde_json::value::to_value(playlist).unwrap()), None).await)
                    },
                    None => Ok(generate_response(StatusCode::NoContent, None::<Value>, None).await)
                }
            }

        },
        Err(_) => Ok(generate_response(StatusCode::BadRequest, None::<Value>, None).await)
    }
}

/// Main function
#[async_std::main] 
async fn main() -> tide::Result<()> {
    let mut db_client_options = match ClientOptions::parse(&*CONNECTION_STRING). await {
        Ok(client_options) => client_options,
        Err(e) => panic!("Failed to generate client options! Reason: {}", e),
    };
    db_client_options.app_name = Some(String::from("Guava"));

    let db_client = match Client::with_options(db_client_options) {
        Ok(client) => client,
        Err(e) => panic!("Failed to open connection to database! Reason: {}", e),
    };

    let state: State = State { db: db_client.database("guava") };
    
    let mut app = tide::with_state(state);

    // list playlists
    app.at("/playlist").get(list_playlist);
    app.at("/playlist/:name/content").get(get_playlist_content);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}