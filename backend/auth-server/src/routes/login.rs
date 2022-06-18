use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use argonautica::Verifier;
use sea_orm::{entity::*, QueryFilter};

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

use entity::auth;

#[derive(Deserialize, Serialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct Claim {
    pub user_id: i64,
    pub username: String,
    pub password_version: f64,
    pub exp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub jwt: String,
}

pub async fn login(form: web::Json<LoginData>, db: web::Data<crate::AppState>) -> impl Responder {
    let secret = std::env::var("PASSWORD_SECRET_KEY").unwrap();

    let user = auth::Entity::find()
        .filter(auth::Column::Username.eq(form.username.clone()))
        .one(&db.conn)
        .await
        .unwrap();

    match user {
        Some(user) => {
            let valid = Verifier::default()
                .with_hash(user.user_password.clone())
                .with_password(form.password.clone())
                .with_secret_key(secret)
                .verify()
                .unwrap();

            if valid {
                let token = Token {
                    jwt: generate_token(user),
                };
                HttpResponse::Ok().json(token)
            } else {
                HttpResponse::Forbidden().json("Invalid password")
            }
        }
        None => {
            return HttpResponse::NotFound().json("User not found");
        }
    }
}

fn generate_token(user: auth::Model) -> String {
    let key = std::env::var("AUTH_SECRET_KEY").expect("SECRET_KEY must be set");

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(60))
        .expect("valid timestamp")
        .timestamp();

    let claim = Claim {
        user_id: user.id,
        username: user.username,
        password_version: user.password_version,
        exp: expiration as usize,
    };
    let token = encode(
        &Header::new(Algorithm::HS512),
        &claim,
        &EncodingKey::from_secret(key.as_ref()),
    );

    token.unwrap()
}
