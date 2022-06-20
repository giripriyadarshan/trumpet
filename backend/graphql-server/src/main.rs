#[macro_use]
extern crate juniper;

use actix_web::{
    http::Error, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_lab::respond::Html;
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use migration::{Migrator, MigratorTrait};
use schemas::root::Context;
// use entity::*;

// use sea_orm::{entity::*, query::*, DatabaseConnection};
// use serde::{Deserialize, Serialize};

// use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
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
        Err(_) => "0.0.0.0:8000".to_string(),
    };

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection = sea_orm::Database::connect(&db_url).await.unwrap();

    Migrator::up(&connection, None).await.unwrap();
    let state = Context { connection };

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(schemas::root::create_schema()))
            .service(web::resource("/").name("home").route(web::get().to(index)))
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql))
                    .route(web::get().to(graphql)),
            )
            .service(web::resource("/graphiql").route(web::get().to(graphql_playground)))
    })
    .bind_openssl(host_address, builder)?
    .run()
    .await
}

async fn graphql(
    pool: web::Data<Context>,
    schema: web::Data<schemas::root::Schema>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let ctx = Context {
        connection: pool.connection.to_owned(),
    };

    let res = data.execute(&schema, &ctx).await;

    Ok(HttpResponse::Ok().json(res))
}

async fn graphql_playground() -> impl Responder {
    Html(graphiql_source("/graphql", None))
}
