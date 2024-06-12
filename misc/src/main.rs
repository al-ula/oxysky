use oxysky_lib::{
    server::{CreateSession, Session},
    Response,
};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let secret: Value =
        serde_json::from_str(tokio::fs::read_to_string(".secret").await.unwrap().as_str()).unwrap();
    let mut id = secret["identifier"].as_str().unwrap().to_string();
    let mut pass = secret["password"].as_str().unwrap().to_string();
    let mut session = Session::default();
    let client = reqwest::Client::new();
    let create_session_url = "https://bsky.social/xrpc/com.atproto.server.createSession";
    let create_session = CreateSession::new(id.clone(), pass.clone(), None);
    let try_session = create_session.send(&client, create_session_url).await;
    match try_session {
        Ok(r) => match r {
            Response::Ok(s) => {
                session = s;
                println!("Success: {}", session.did);
            }
            Response::Err(no_session) => {
                println!("Error: {}", no_session.error);
                println!("Message: {}", no_session.message);
            }
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
    let session_info = session
        .get(
            &client,
            "https://bsky.social/xrpc/com.atproto.server.getSession",
        )
        .await
        .unwrap();

    println!("Session Info:\n{:?}", session_info);
}
