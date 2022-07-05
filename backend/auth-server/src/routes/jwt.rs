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

    let get_auth = auth::Entity::find()
        .filter(auth::Column::Username.eq(form.username.clone()))
        .one(&db.conn)
        .await
        .unwrap();

    return match get_auth {
        Some(auth) => {
            let valid = Verifier::default()
                .with_hash(auth.user_password.clone())
                .with_password(form.password.clone())
                .with_secret_key(secret)
                .verify()
                .unwrap();

            if valid {
                let token: Token;
                let user_id: i64 = entity::users::Entity::find()
                    .filter(entity::users::Column::AuthId.eq(auth.id))
                    .one(&db.conn)
                    .await
                    .unwrap()
                    .unwrap()
                    .id;
                if path.as_str() == "login" {
                    token = Token {
                        jwt: generate_token(auth, user_id, RequestType::Login),
                        user_id,
                    };
                } else if path.as_str() == "one-time-jwt" {
                    token = Token {
                        jwt: generate_token(auth, user_id, RequestType::OneTimeJwt),
                        user_id,
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

fn generate_token(auth: auth::Model, user_id: i64, request_type: RequestType) -> String {
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
        auth_id: auth.id,
        user_id,
        username: auth.username,
        password_version: auth.password_version,
        exp: expiration as usize,
    };
    let token = encode(
        &Header::new(Algorithm::HS512),
        &claim,
        &EncodingKey::from_secret(key.as_ref()),
    );

    token.unwrap()
}
