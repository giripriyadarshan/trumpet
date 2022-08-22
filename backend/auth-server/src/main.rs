use actix_web::{middleware::Logger, web, App, HttpServer, http};
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use futures::future;

use actix_cors::Cors;

use sea_orm::DatabaseConnection;

mod models;
mod routes;

#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let internal_builder = ssl_builder();
    let external_builder = ssl_builder();

    let external_address = match std::env::var("AUTH_EXTERNAL_SERVER_HOST") {
        Ok(host) => host,
        Err(_) => "0.0.0.0:9000".to_string(),
    };

    let internal_address = match std::env::var("AUTH_INTERNAL_SERVER_HOST") {
        Ok(host) => host,
        Err(_) => "0.0.0.0:9004".to_string(),
    };

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection = sea_orm::Database::connect(&db_url).await.unwrap();

    let internal_state = AppState {
        conn: connection.clone(),
    };
    let external_state = AppState { conn: connection };

    let external_server = HttpServer::new(move || {

        let cors = Cors::default()
            .allowed_origin("http://localhost/")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b".localhost")
            })
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(external_state.clone()))
            .route("/jwt/{type}", web::post().to(routes::jwt::jwt))
    })
    .bind_openssl(external_address, external_builder)?
    .run();

    let internal_server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(internal_state.clone()))
            .route(
                "/authenticate",
                web::post().to(routes::authenticate::authenticate),
            )
    })
    .bind_openssl(internal_address, internal_builder)?
    .run();

    future::try_join(external_server, internal_server).await?;

    Ok(())
}

fn ssl_builder() -> SslAcceptorBuilder {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("192.168.0.108+3-key.pem", SslFiletype::PEM)
        .unwrap();

    builder
        .set_certificate_chain_file("192.168.0.108+3.pem")
        .unwrap();

    builder
}
