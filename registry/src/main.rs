use std::{fs, path::PathBuf, net::SocketAddr};
use axum::{
    extract::{Path},
    routing::{get},
    Json, Router,
};
use tokio::{fs::File};
use tokio_util::io::ReaderStream;
use anyhow::Result;

const STORAGE_DIR: &str = "./storage";
const LOADER_DIR: &str = "./loader";

#[tokio::main]
async fn main() -> Result<()> {
    fs::create_dir_all(STORAGE_DIR)?;

    let app = Router::new()
        .route("/download/{id}", get(download_package))
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
