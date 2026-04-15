use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures_util::StreamExt;
use serde::Serialize;
use tracing_actix_web::TracingLogger;

use sofdocs_core::document::parser::parse_docx;
use sofdocs_core::document::renderer::render_to_html;
use sofdocs_core::document::writer::write_docx;

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

async fn convert_docx_to_html(mut payload: Multipart) -> impl Responder {
    let mut bytes = Vec::new();

    while let Some(Ok(mut field)) = payload.next().await {
        while let Some(Ok(chunk)) = field.next().await {
            bytes.extend_from_slice(&chunk);
        }
    }

    if bytes.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "No file uploaded"}));
    }

    match parse_docx(&bytes) {
        Ok(doc) => {
            let html = render_to_html(&doc);
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn export_docx(mut payload: Multipart) -> impl Responder {
    let mut bytes = Vec::new();

    while let Some(Ok(mut field)) = payload.next().await {
        while let Some(Ok(chunk)) = field.next().await {
            bytes.extend_from_slice(&chunk);
        }
    }

    if bytes.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "No file uploaded"}));
    }

    match parse_docx(&bytes) {
        Ok(doc) => match write_docx(&doc) {
            Ok(docx_bytes) => HttpResponse::Ok()
                .content_type("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
                .append_header(("Content-Disposition", "attachment; filename=\"document.docx\""))
                .body(docx_bytes),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        },
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
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
                    .route("/documents/upload", web::post().to(upload_document))
                    .route("/convert", web::post().to(convert_docx_to_html))
                    .route("/export", web::post().to(export_docx)),
            )
    })
    .bind(&bind)?
    .run()
    .await
}
