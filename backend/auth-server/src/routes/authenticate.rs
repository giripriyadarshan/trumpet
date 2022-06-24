use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;

use sea_orm::entity::*;

use entity::auth;

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use chrono::Utc;

use crate::models::{Claim, Token};

#[derive(Serialize)]
struct AuthenticationStatus {
    user_id: i64,
    username: String,
    is_authenticated: bool,
    exp: bool,
}

pub async fn authenticate(
    form: web::Json<Token>,
    db: web::Data<crate::AppState>,
) -> impl Responder {
    let key = std::env::var("AUTH_SECRET_KEY").unwrap();

    let token = form.jwt.clone();

    let token = decode::<Claim>(
        token.as_str(),
        &DecodingKey::from_secret(key.as_ref()),
        &Validation::new(Algorithm::HS512),
    )
    .map_err(|e| e.to_string());

    match token {
        Ok(token) => {
            let token = token.claims;
            let user = auth::Entity::find_by_id(token.user_id)
                .one(&db.conn)
                .await
                .unwrap()
                .unwrap();

            let time_now = Utc::now().timestamp() as usize;
            let time_exp = token.exp;

            HttpResponse::Ok().json(AuthenticationStatus {
                user_id: user.id,
                is_authenticated: user.username == token.username
                    && user.password_version == token.password_version,
                username: user.username,
                exp: time_exp - time_now < 120,
            })
        }

        Err(e) => HttpResponse::Unauthorized().json(e),
    }
}
