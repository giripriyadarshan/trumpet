use actix_web::{middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use sea_orm::DatabaseConnection;

mod routes;

#[derive(Debug, Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("192.168.0.108+3-key.pem", SslFiletype::PEM)
        .unwrap();

    builder
        .set_certificate_chain_file("192.168.0.108+3.pem")
        .unwrap();

    let host_address = match std::env::var("AUTH_SERVER_HOST") {
        Ok(host) => host,
        Err(_) => "0.0.0.0:9000".to_string(),
    };

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection = sea_orm::Database::connect(&db_url).await.unwrap();

    let state = AppState { conn: connection };

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(state.clone()))
            .route("/", web::get().to(index))
            .route("/login", web::post().to(routes::login::login))
            .route(
                "/authenticate",
                web::post().to(routes::authenticate::authenticate),
            )
    })
    .bind_openssl(host_address, builder)?
    .run()
    .await
}

async fn index(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json(routes::login::LoginData {
        username: "admin".to_string(),
        password: "admin".to_string(),
    })
}
