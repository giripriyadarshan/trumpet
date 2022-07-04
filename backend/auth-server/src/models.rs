use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct Claim {
    pub auth_id: i64,
    pub user_id: i64,
    pub username: String,
    pub password_version: f64,
    pub exp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub jwt: String,
}

pub enum RequestType {
    Login,
    OneTimeJwt,
}
