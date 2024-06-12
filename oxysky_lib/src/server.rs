use header::AUTHORIZATION;
use reqwest::Result;
use serde::{Deserialize, Serialize};
use serde_json;

use super::{Response, *};

#[derive(Debug, Default, Clone, Serialize)]
pub struct CreateSession {
    pub identifier: String,
    pub password: String,
    #[serde(rename = "authFactorToken")]
    pub auth_factor_token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(default, rename = "accessJwt")]
    pub access_jwt: Bearer,
    #[serde(default, rename = "refreshJwt")]
    pub refresh_jwt: Bearer,
    #[serde(default)]
    pub handle: Handle,
    #[serde(default)]
    pub did: DID,
    #[serde(default, rename = "didDoc")]
    pub did_doc: DidDoc,
    #[serde(default)]
    pub email: String,
    #[serde(default, rename = "emailConfirmed")]
    pub email_confirmed: bool,
    #[serde(default, rename = "emailAuthFactor")]
    pub email_auth_factor: bool,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DidDoc {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Vec<String>,
    pub id: String,
    pub service: Vec<Service>,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
    #[serde(rename = "type")]
    pub service_type: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub controller: String,
    pub id: String,
    #[serde(rename = "publicKeyMultibase")]
    pub public_key_multibase: String,
    #[serde(rename = "type")]
    pub verif_type: String,
}

impl CreateSession {
    pub fn new(id: String, pass: String, auth_factor: Option<String>) -> Self {
        Self {
            identifier: (id),
            password: (pass),
            auth_factor_token: (auth_factor),
        }
    }

    pub async fn send(&self, client: &Client, url: &str) -> Result<Response<Session>> {
        let payload: CreateSession = match self.auth_factor_token {
            Some(_) => CreateSession {
                identifier: self.identifier.clone(),
                password: self.password.clone(),
                auth_factor_token: self.auth_factor_token.clone(),
            },
            None => CreateSession {
                identifier: self.identifier.clone(),
                password: self.password.clone(),
                auth_factor_token: Some(String::new()),
            },
        };
        let body = serde_json::to_string(&payload).unwrap();
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        let resp = client.post(url).headers(headers).body(body).send().await;
        match resp {
            Err(e) => Err(e),
            Ok(r) => match r.status() {
                StatusCode::OK => {
                    let session =
                        serde_json::from_str::<Session>(&r.text().await.unwrap()).unwrap();
                    Ok(Response::Ok(session))
                }
                StatusCode::BAD_REQUEST => {
                    let no_session = serde_json::from_str::<NO>(&r.text().await.unwrap()).unwrap();
                    Ok(Response::Err(no_session))
                }
                _ => Ok(Response::Err(NO::default())),
            },
        }
    }
}

impl Session {
    pub fn new(
        access_jwt: Bearer,
        refresh_jwt: Bearer,
        handle: Handle,
        did: DID,
        did_doc: DidDoc,
        email: String,
        email_confirmed: bool,
        email_auth_factor: bool,
        active: bool,
        status: Option<String>,
    ) -> Self {
        Self {
            access_jwt,
            refresh_jwt,
            handle,
            did,
            did_doc,
            email,
            email_confirmed,
            email_auth_factor,
            active,
            status,
        }
    }

    pub async fn refresh(&self, client: &Client, url: &str) -> Result<Response<Session>> {
        println!("Refresh JWT: {}", self.refresh_jwt);
        let refresh = refresh_session(client, url, &self.refresh_jwt.clone()).await;
        let mut refreshed: Session = self.clone();
        match refresh {
            Ok(r) => match r {
                Response::Ok(s) => {
                    refreshed.access_jwt = s.access_jwt;
                    refreshed.refresh_jwt = s.refresh_jwt;
                    refreshed.handle = s.handle;
                    refreshed.did = s.did;
                    refreshed.did_doc = s.did_doc;
                    refreshed.active = s.active;
                    refreshed.status = s.status;
                    Ok(Response::Ok(refreshed))
                }
                Response::Err(no_session) => Ok(Response::Err(no_session)),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn delete(&self, client: &Client, url: &str) -> Result<Response<StatusCode>> {
        let auth = self.access_jwt.clone();
        delete_session(client, url, auth).await
    }

    pub async fn get(&self, client: &Client, url: &str) -> Result<Response<Session>> {
        let auth = self.access_jwt.clone();
        get_session(client, url, auth).await
    }
}

impl DidDoc {
    pub fn new(
        context: Vec<String>,
        also_known_as: Vec<String>,
        id: String,
        service: Vec<Service>,
        verification_method: Vec<VerificationMethod>,
    ) -> Self {
        Self {
            context,
            also_known_as,
            id,
            service,
            verification_method,
        }
    }
}

impl Service {
    pub fn new(id: String, service_endpoint: String, service_type: String) -> Self {
        Self {
            id,
            service_endpoint,
            service_type,
        }
    }
}

impl VerificationMethod {
    pub fn new(
        controller: String,
        id: String,
        public_key_multibase: String,
        verif_type: String,
    ) -> Self {
        Self {
            controller,
            id,
            public_key_multibase,
            verif_type,
        }
    }
}

pub async fn refresh_session(
    client: &Client,
    url: &str,
    refresh: &Bearer,
) -> Result<Response<Session>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", refresh)).unwrap(),
    );
    let resp = client.get(url).headers(headers).send().await;
    println!("Refresh Response: {:?}", resp);
    match resp {
        Err(e) => Err(e),
        Ok(r) => match r.status() {
            StatusCode::OK => {
                let session = serde_json::from_str::<Session>(&r.text().await.unwrap()).unwrap();
                Ok(Response::Ok(session))
            }
            StatusCode::BAD_REQUEST => {
                let no_session = serde_json::from_str::<NO>(&r.text().await.unwrap()).unwrap();
                Ok(Response::Err(no_session))
            }
            StatusCode::UNAUTHORIZED => {
                let no_session = serde_json::from_str::<NO>(&r.text().await.unwrap()).unwrap();
                Ok(Response::Err(no_session))
            }
            _ => Ok(Response::Err(NO::default())),
        },
    }
}

pub async fn delete_session(
    client: &Client,
    url: &str,
    auth: Bearer,
) -> Result<Response<StatusCode>> {
    let auth = "Bearer ".to_string() + &auth.to_string();
    let resp = client.get(url).header(AUTHORIZATION, auth).send().await;
    match resp {
        Err(e) => Err(e),
        Ok(r) => match r.status() {
            StatusCode::OK => Ok(Response::Ok(StatusCode::OK)),
            StatusCode::UNAUTHORIZED => {
                let no_session = serde_json::from_str::<NO>(&r.text().await.unwrap()).unwrap();
                Ok(Response::Err(no_session))
            }
            StatusCode::BAD_REQUEST => {
                let no_session = serde_json::from_str::<NO>(&r.text().await.unwrap()).unwrap();
                Ok(Response::Err(no_session))
            }
            _ => Ok(Response::Err(NO::default())),
        },
    }
}

pub async fn get_session(client: &Client, url: &str, auth: Bearer) -> Result<Response<Session>> {
    let auth = "Bearer ".to_string() + &auth.to_string();
    let resp = client.get(url).header(AUTHORIZATION, auth).send().await;
    match resp {
        Err(e) => Err(e),
        Ok(r) => match r.status() {
            StatusCode::OK => {
                let session = serde_json::from_str::<Session>(&r.text().await.unwrap()).unwrap();
                Ok(Response::Ok(session))
            }
            StatusCode::UNAUTHORIZED => {
                let no_session = serde_json::from_str::<NO>(&r.text().await.unwrap()).unwrap();
                Ok(Response::Err(no_session))
            }
            _ => Ok(Response::Err(NO::default())),
        },
    }
}
