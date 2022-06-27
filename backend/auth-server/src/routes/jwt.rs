use actix_web::{web, HttpResponse, Responder};

use argonautica::Verifier;
use sea_orm::{entity::*, QueryFilter};

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

use entity::auth;

use crate::models::{Claim, LoginData, RequestType, Token};

pub async fn jwt(
    form: web::Json<LoginData>,
    db: web::Data<crate::AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let secret = std::env::var("PASSWORD_SECRET_KEY").unwrap();

    let user = auth::Entity::find()
        .filter(auth::Column::Username.eq(form.username.clone()))
        .one(&db.conn)
        .await
        .unwrap();

    return match user {
        Some(user) => {
            let valid = Verifier::default()
                .with_hash(user.user_password.clone())
                .with_password(form.password.clone())
                .with_secret_key(secret)
                .verify()
                .unwrap();

            if valid {
                let token: Token;
                if path.as_str() == "login" {
                    token = Token {
                        jwt: generate_token(user, RequestType::Login),
                    };
                } else if path.as_str() == "one-time-jwt" {
                    token = Token {
                        jwt: generate_token(user, RequestType::OneTimeJwt),
                    };
                } else {
                    return HttpResponse::ServiceUnavailable().finish();
                }
                HttpResponse::Ok().json(token)
            } else {
                HttpResponse::Forbidden().json("Invalid password")
            }
        }
        None => HttpResponse::NotFound().json("User not found"),
    };
}

fn generate_token(user: auth::Model, request_type: RequestType) -> String {
    let key = std::env::var("AUTH_SECRET_KEY").expect("SECRET_KEY must be set");

    let expiration = match request_type {
        RequestType::Login => chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(60))
            .expect("valid timestamp")
            .timestamp(),
        RequestType::OneTimeJwt => chrono::Utc::now()
            .checked_add_signed(chrono::Duration::minutes(1))
            .expect("valid timestamp")
            .timestamp(),
    };

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
