use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

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

    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .bind_openssl(host_address, builder)?
        .run()
        .await
}

async fn index(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
