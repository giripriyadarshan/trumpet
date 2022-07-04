use reqwest;
use std::collections::HashMap;

pub struct Authenticated {
    pub auth_id: i64,
    pub user_id: i64,
    pub username: String,
    pub is_one_time_jwt: bool,
}

pub enum AuthenticationStatus {
    Authenticated(Authenticated),
    Unauthenticated,
}

pub async fn authenticate(jwt: String) -> AuthenticationStatus {
    let mut json_jwt = HashMap::new();
    json_jwt.insert("jwt", jwt);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let res = client
        .post("https://localhost:9004/authenticate")
        .json(&json_jwt)
        .send()
        .await;

    match res {
        Ok(res) => {
            let status = res.status();
            if status == 200 {
                let json: serde_json::Value = res.json().await.unwrap();
                if json["is_authenticated"].as_bool().unwrap() {
                    AuthenticationStatus::Authenticated(Authenticated {
                        auth_id: json["auth_id"].as_i64().unwrap(),
                        user_id: json["user_id"].as_i64().unwrap(),
                        username: json["username"].as_str().unwrap().to_string(),
                        is_one_time_jwt: json["is_one_time_jwt"].as_bool().unwrap(),
                    })
                } else {
                    AuthenticationStatus::Unauthenticated
                }
            } else {
                AuthenticationStatus::Unauthenticated
            }
        }
        Err(_) => AuthenticationStatus::Unauthenticated,
    }
}
