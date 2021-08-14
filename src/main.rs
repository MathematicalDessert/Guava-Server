pub mod service;

use std::env;
use async_std::{path::PathBuf};
use serde_json::{self, Map, Value};
use tide::{Body, Request, Response, StatusCode, prelude::*};
use lazy_static::lazy_static;
use futures::{stream::TryStreamExt};
use mongodb::{Client, Collection, bson::doc, options::{ClientOptions}};
use crate::service::content_service::{ContentService, GuavaContentType};

lazy_static! {
    static ref MONGO_HOST: String = env::var("MONGO_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    static ref MONGO_PORT: String = env::var("MONGO_PORT").unwrap_or_else(|_| "27017".to_string());
    static ref CONNECTION_STRING: String = format!("mongodb://{}:{}/", MONGO_HOST.as_str(), MONGO_PORT.as_str());
}

#[derive(Clone)]
struct State {
    db: mongodb::Database,
    content_service: ContentService,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlaylistContent {
    name: String,
    content_type: GuavaContentType,
    content_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GuavaPlaylist {
    name: String,
    identifier: String,
    content: Option<Vec<PlaylistContent>>
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

/// List playlist
/// 
/// Lists names of all known playlists
async fn list_playlist(req: Request<State>) -> tide::Result {
    let db = &req.state().db;
    let playlist_collection: Collection<GuavaPlaylist> = db.collection("playlist"); 

    let results = playlist_collection.find(None, None).await?;
    let playlists = results.try_collect().await.unwrap_or_else(|_| vec![]);
    
    Ok(generate_response(StatusCode::Ok, Some(serde_json::value::to_value(playlists).unwrap()), None).await)
}

async fn download_asset(req: Request<State>) -> tide::Result {
    let content_service  = &req.state().content_service;

    match content_service.get_hash_from_id(req.param("id").unwrap().to_string()).await {
        Ok(hash) => {
            match Body::from_file(PathBuf::from("content/".to_string().to_owned() + &hash.to_owned())).await {
                Ok(body) => Ok(Response::builder(StatusCode::Ok).body(body).build()),
                Err(_) => Ok(generate_response(StatusCode::NotFound, None::<Value>, Some(String::from("file not found"))).await)
            }
        },
        Err(_) => Ok(generate_response(StatusCode::NotFound, None::<Value>, Some(String::from("file not found"))).await),
    }
}

async fn get_hash_of_content(req: Request<State>) -> tide::Result {
    let content_service = &req.state().content_service;
    
    match content_service.get_hash_from_id(req.param("id").unwrap().to_string()).await {
        Ok(hash) => Ok(generate_response(StatusCode::Ok, Some(serde_json::Value::String(String::from(hash))), None).await),
        Err(_) => Ok(generate_response(StatusCode::NotFound, None::<Value>, Some(String::from("content not found"))).await),
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

    let state: State = State { 
        db: db_client.database("guava"),
        content_service: ContentService::new(db_client.database("guava"))
    };
    
    let mut app = tide::with_state(state);
    app.with(tide::log::LogMiddleware::new()); 

    tide::log::start();

    // list playlists

    // index
    app.at("/").get(|_| async move { Ok(String::from("OK")) });

    // content
    app.at("/content/:id/hash").get(get_hash_of_content);
    app.at("/content/:id/download").get(download_asset);

    // playlist
    app.at("/playlists").get(list_playlist);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}