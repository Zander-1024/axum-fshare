use axum::{
    extract::Query,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;
#[derive(Clone, Debug)]
struct Data {
    data: Arc<Mutex<HashMap<String, String>>>,
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
#[derive(Serialize)]
struct Url {
    url: String,
}
#[derive(Deserialize)]
struct SubjectArgs {
    uuid: String,
}
// 请求主体是一个异步流，只能使用一次。因此，您只能有一个提取器来消耗请求正文。
// axum 通过要求此类提取器作为处理程序采用的最后一个参数来强制执行此操作。
//https://docs.rs/axum/latest/axum/extract/index.html#the-order-of-extractors
async fn generate_url(Extension(data): Extension<Data>, Json(file): Json<FilePath>) -> Json<Url> {
    let uuid = Uuid::new_v4().to_string();
    if let Ok(mut map) = data.data.lock() {
        map.insert(uuid.clone(), file.path);
        println!("HashMap contents:");
        for (key, value) in map.iter() {
            println!("Key: {}, Value: {}", key, value);
        }
    }
    let ans = Url {
        url: format!("/get_file?uuid={}", uuid),
    };
    Json(ans)
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

async fn get_data(Extension(data): Extension<Data>, Query(args): Query<SubjectArgs>) -> String {
    if let Ok(map) = data.data.lock() {
        println!("HashMap contents:");
        for (key, value) in map.iter() {
            println!("Key: {}, Value: {}", key, value);
            println!("uuid: {}", &args.uuid);
        }
        match map.get(&args.uuid) {
            Some(path) => path.clone(),
            None => String::from("Data not found"),
        }
    } else {
        String::from("Data not found")
    }
}

#[tokio::main]
async fn main() {
    let data = Data::new();

    let app = Router::new()
        .route("/generate_url", post(generate_url))
        .route("/get_file", get(get_data))
        .layer(Extension(data));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
