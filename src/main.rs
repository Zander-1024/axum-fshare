use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use local_ip_address::local_ip;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use clap::Parser;
use qrrs::qrcode;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the file or folder to be shared
    path: Option<String>,

    /// Service Port. Default 9527
    #[arg(short, long)]
    port: Option<u16>,
}
#[derive(Clone, Debug)]
struct Data {
    data: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Clone, Debug)]
struct LocalIp {
    ip: IpAddr,
    port: u16,
}

impl Data {
    fn new() -> Self {
        Data {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
// the input to our `create_user` handler
#[derive(Deserialize)]
struct FilePath {
    path: String,
}

// 请求主体是一个异步流，只能使用一次。因此，您只能有一个提取器来消耗请求正文。
// axum 通过要求此类提取器作为处理程序采用的最后一个参数来强制执行此操作。
//https://docs.rs/axum/latest/axum/extract/index.html#the-order-of-extractors
async fn generate_url(
    Extension(data): Extension<Data>,
    Extension(local_ip): Extension<LocalIp>,
    Json(file): Json<FilePath>,
) -> Result<String, StatusCode> {
    generate_url_sync(&data, &local_ip, &file.path)
}

fn generate_url_sync(data: &Data, local_ip: &LocalIp, path: &str) -> Result<String, StatusCode> {
    let file_path = path::Path::new(path);
    if file_path.exists() {
        let uuid = Uuid::new_v4().to_string();
        if let Ok(mut map) = data.data.lock() {
            map.insert(uuid.clone(), path.to_owned());
        }
        let file_name = file_path.file_name().unwrap();

        Ok(format!(
            "http://{:?}:{}/get_file/{}/{}",
            local_ip.ip,
            local_ip.port,
            uuid,
            file_name.to_str().unwrap()
        ))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
// 在 Axum 中，您可以使用不同的方式来获取 GET 请求中的参数。以下是一些常见的方法：

// Path 参数：
// Path 参数，也称为路径参数，是直接从 URL 路径中提取的参数。
// 您可以将 URL 的一部分变成参数化，以便动态地处理不同的请求。
// 示例：GET /user/:id，其中 :id 是一个 Path 参数。
// 使用 axum::extract::Path 可以方便地获取 Path 参数。
// URL 参数：
// URL 参数是附加在 URL 后面的键值对，以 ? 开头，多个参数之间使用 & 分隔。
// 示例：GET /subject?page=1&keyword=axum.rs
// 使用 axum::extract::Query 可以获取 URL 参数。

async fn get_data(
    Extension(data): Extension<Data>,
    Path((uuid, _team_id)): Path<(String, String)>,
) -> Result<Vec<u8>, StatusCode> {
    if let Ok(map) = data.data.lock() {
        match map.get(&uuid) {
            Some(path) => match std::fs::read(path) {
                Ok(file) => Ok(file),
                Err(_) => Err(StatusCode::NOT_FOUND),
            },
            None => Err(StatusCode::NOT_FOUND),
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[tokio::main]
async fn main() {
    let data = Data::new();
    let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let mut port = 9527;
    let my_local_ip = local_ip().unwrap();

    let cli = Cli::parse();
    if let Some(p) = cli.port {
        port = p;
    }
    println!("This is my local IP address: {:?}:{}", my_local_ip, port);
    let local_ip_port = LocalIp {
        ip: my_local_ip,
        port,
    };
    // You can check the value provided by positional arguments, or option arguments
    if let Some(path) = cli.path.as_deref() {
        println!("Value for path: {:?}", path);
        match generate_url_sync(&data, &local_ip_port, path) {
            Ok(url) => match qrcode::make_code(&url) {
                Ok(code) => qrcode::print_code_to_term(
                    &code,
                    qrcode::QrCodeViewArguments {
                        margin: 5,
                        invert_colors: false,
                    },
                ),
                Err(_) => println!("Failed to generate a QR code"),
            },
            Err(err) => println!("err code: {:?}", err),
        }
    }

    let app = Router::new()
        .route("/generate_url", post(generate_url))
        .route("/get_file/:uuid/:file_name", get(get_data))
        .layer(Extension(data))
        .layer(Extension(local_ip_port));

    let socket_addr = SocketAddr::new(ip, port);

    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
