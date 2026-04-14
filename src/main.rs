use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use tracing_actix_web::TracingLogger;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Serialize)]
struct DocumentListResponse {
    documents: Vec<serde_json::Value>,
}

async fn list_documents() -> impl Responder {
    HttpResponse::Ok().json(DocumentListResponse {
        documents: vec![],
    })
}

async fn upload_document() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Upload endpoint — not yet implemented"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    tracing::info!("Starting SofDocs server on {bind}");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api")
                    .route("/documents", web::get().to(list_documents))
                    .route("/documents/upload", web::post().to(upload_document)),
            )
    })
    .bind(&bind)?
    .run()
    .await
}
