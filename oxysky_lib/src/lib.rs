use header;
pub use reqwest::StatusCode;
use reqwest::*;
use serde::{Deserialize, Serialize};
pub use serde_json::json;
pub type Bearer = String;
pub type Handle = String;
pub type DID = String;
pub mod server;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response<T> {
    Ok(T),
    Err(NO),
}
impl<T> Response<T> {
    pub fn unwrap(self) -> T {
        match self {
            Response::Ok(value) => value,
            Response::Err(_) => panic!("called `Response::unwrap()` on an `Err` value"),
        }
    }
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NO {
    pub error: String,
    pub message: String,
}
impl NO {
    pub fn new(error: String, message: String) -> Self {
        Self { error, message }
    }
}
