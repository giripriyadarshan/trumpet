use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

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

    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .bind_openssl(host_address, builder)?
        .run()
        .await
}
