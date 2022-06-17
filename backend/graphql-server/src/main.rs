#[macro_use]
extern crate juniper;

use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use entity::*;
use migration::{Migrator, MigratorTrait};

use sea_orm::{entity::*, query::*, DatabaseConnection};
use serde::{Deserialize, Serialize};

use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
mod lib;
mod schemas;

async fn index(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("192.168.0.108+3-key.pem", SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file("192.168.0.108+3.pem")
        .unwrap();

    let host_address = match std::env::var("HOST_ADDRESS") {
        Ok(host_address) => host_address,
        Err(_) => "127.0.0.1:8000".to_string(),
    };

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(connection.clone())
            .service(web::resource("/").name("home").route(web::get().to(index)))
    })
    .bind_openssl(host_address, builder)?
    .run()
    .await
}
