use crate::cli::Cli;
use anyhow::Result;
use std::path::PathBuf;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use axum::{
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::header,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};

use local_ip_address::local_ip;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path;
use std::sync::Mutex;

use uuid::Uuid;

use qrrs::qrcode::{self, QrCodeViewArguments};

type DB = Arc<RwLock<HashMap<Uuid, PathBuf>>>;

pub fn build_app(args: &Cli) -> Result<Router> {
    let db = DB::default();
    let my_local_ip = local_ip().unwrap();
    let local_ip_port = format!("{}:{}", my_local_ip, args.port);
    // You can check the value provided by positional arguments, or option arguments
    generate_qr_code(args, &local_ip_port, db.clone());

    let app = Router::new()
        .route("/generate_url", post(generate_url))
        .route("/get_file/:uuid/:file_name", get(get_data))
        .layer(Extension(local_ip_port))
        .with_state(db);

    Ok(app)
}

fn generate_qr_code(args: &Cli, local_ip_port: &str, db: DB) {
    if let Some(path) = &args.path {
        println!("Value for path: {:?}", path);
        match generate_url_sync(db, &local_ip_port, &path) {
            Ok(url) => print_qrcode_to_term(url, args.into()),
            Err(err) => println!("err code: {:?}", err),
        }
    }
}
fn generate_url_sync(db: DB, local_ip: &str, file_path: &PathBuf) -> Result<String, StatusCode> {
    if file_path.exists() {
        let uuid = Uuid::new_v4();
        if let Ok(mut map) = db.write() {
            map.insert(uuid.clone(), file_path.to_owned());
        }
        let file_name = file_path.file_name().unwrap();
        let out_url = format!("http://{}/get_file/{:?}/{:?}", local_ip, uuid, file_name);
        println!("url: {}", &out_url);
        Ok(out_url)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn print_qrcode_to_term(url: String, view_args: QrCodeViewArguments) {
    match qrcode::make_code(&url) {
        Ok(code) => qrcode::print_code_to_term(&code, view_args),
        Err(_) => println!("Failed to generate a QR code"),
    }
}
// the input to our `create_user` handler
#[derive(Deserialize)]
struct FilePath {
    path: PathBuf,
}

// 请求主体是一个异步流，只能使用一次。因此，您只能有一个提取器来消耗请求正文。
// axum 通过要求此类提取器作为处理程序采用的最后一个参数来强制执行此操作。
//https://docs.rs/axum/latest/axum/extract/index.html#the-order-of-extractors
async fn generate_url(
    State(data): State<DB>,
    Extension(local_ip): Extension<&str>,
    Json(file): Json<FilePath>,
) -> Result<String, StatusCode> {
    generate_url_sync(data, &local_ip, &file.path)
}

#[debug_handler]
async fn get_data(
    State(data): State<DB>,
    Path((uuid, file_name)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    let file_path = {
        match data.read() {
            Ok(map) => {
                let file = map.get(&uuid).unwrap();
                Some(file.clone())
            }
            Err(_) => None,
        }
    };

    match file_path {
        None => (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8".to_string()),
                (header::SERVER, "axum".to_string()),
            ],
            Body::from("invalid path".to_string()),
        ),
        Some(path) => {
            let file = tokio::fs::File::open(path).await.unwrap();
            let stream = tokio_util::io::ReaderStream::new(file);
            let body = Body::from_stream(stream);

            let headers = [
                (
                    header::CONTENT_TYPE,
                    "text/plain; charset=utf-8".to_string(),
                ),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", file_name),
                ),
            ];

            (headers, body)
        }
    }
}
