use std::{fs, path::PathBuf, net::SocketAddr};
use axum::{
    extract::{Multipart, Path},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::io::ReaderStream;
use uuid::Uuid;
use anyhow::Result;
use axum::response::Html;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Manifest {
    name: String,
    version: String,
    description: Option<String>,
}

const STORAGE_DIR: &str = "./storage";
const LOADER_DIR: &str = "./loader";

#[tokio::main]
async fn main() -> Result<()> {
    fs::create_dir_all(STORAGE_DIR)?;

    let app = Router::new()
        .route("/upload", post(upload_package))
        .route("/download/{id}", get(download_package))
        .route("/", get(root))
        .route("/packages", get(list_packages))
        .route("/loader/{version}/1", get(download_loader1))
        .route("/loader/{version}/2", get(download_loader2));

    let addr = SocketAddr::from(([0, 0, 0, 0], 7009));
    println!("🚀 mpkg registry running at http://{addr}");
    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app
    )
    .await?;

    Ok(())
}

async fn root() -> Html<String> {
    let content = fs::read_to_string("views/index.html")
        .unwrap_or_else(|_| "<h1>Failed to load file</h1>".to_string());
    
    Html(content)
}

async fn upload_package(mut multipart: Multipart) -> Result<Json<Manifest>, (axum::http::StatusCode, String)> {
    let mut manifest: Option<Manifest> = None;

    while let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        let name = field.name().unwrap_or_default().to_string();
        let package_id = Uuid::new_v4().to_string();
        let package_dir = PathBuf::from(format!("{STORAGE_DIR}/{package_id}"));
        fs::create_dir_all(&package_dir).map_err(internal_error)?;

        if name == "manifest" {
            let data = field.text().await.map_err(internal_error)?;
            manifest = Some(serde_json::from_str(&data).map_err(internal_error)?);
        } else if name == "file" {
            let file_path = package_dir.join("package.zip");
            let mut file = File::create(&file_path).await.map_err(internal_error)?;
            let mut field_data = field;
            while let Some(chunk) = field_data.chunk().await.map_err(internal_error)? {
                file.write_all(&chunk).await.map_err(internal_error)?;
            }
        }
    }

    let manifest = manifest.ok_or((axum::http::StatusCode::BAD_REQUEST, "Missing manifest".to_string()))?;
    println!("📦 Uploaded package: {} v{}", manifest.name, manifest.version);
    Ok(Json(manifest))
}

async fn download_package(Path(id): Path<String>) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    let file_path = PathBuf::from(format!("{STORAGE_DIR}/{id}/package.zip"));
    if !file_path.exists() {
        return Err((axum::http::StatusCode::NOT_FOUND, "Package not found".to_string()));
    }

    let file = File::open(file_path).await.map_err(internal_error)?;
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);
    Ok(axum::response::Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .unwrap())
}
async fn download_loader2(Path(version): Path<String>) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    let file_path = PathBuf::from(format!("{LOADER_DIR}/{version}/mpkg-loader.mjs"));
    if !file_path.exists() {
        return Err((axum::http::StatusCode::NOT_FOUND, "Package not found".to_string()));
    }

    let file = File::open(file_path).await.map_err(internal_error)?;
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);
    Ok(axum::response::Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .unwrap())
}

async fn download_loader1(Path(version): Path<String>) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    let file_path = PathBuf::from(format!("{LOADER_DIR}/{version}/bootstrap.mjs"));
    if !file_path.exists() {
        return Err((axum::http::StatusCode::NOT_FOUND, "Package not found".to_string()));
    }

    let file = File::open(file_path).await.map_err(internal_error)?;
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);
    Ok(axum::response::Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .unwrap())
}

async fn list_packages() -> Result<Json<Vec<String>>, (axum::http::StatusCode, String)> {
    let mut packages = Vec::new();
    for entry in fs::read_dir(STORAGE_DIR).map_err(internal_error)? {
        let entry = entry.map_err(internal_error)?;
        if entry.path().is_dir() {
            packages.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(Json(packages))
}

fn internal_error<E: std::fmt::Display>(err: E) -> (axum::http::StatusCode, String) {
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {err}"))
}
