// http impl in actix-web

use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web::web::Json;
use log::info;
use serde::Deserialize;
use crate::command;
use crate::common::handle;

#[get("/")]
async fn index() -> impl Responder {
    include_str!("../../http-panel/dist/index.html")
        .customize().insert_header(("Content-Type", "text/html"))
}

#[get("/index.html")]
async fn index_html() -> impl Responder {
    include_str!("../../http-panel/dist/index.html")
        .customize().insert_header(("Content-Type", "text/html"))
}

#[get("/favicon.svg")]
async fn favicon() -> impl Responder {
    include_str!("../../http-panel/dist/favicon.svg")
        .customize().insert_header(("Content-Type", "image/svg+xml"))
}

#[get("/index.css")]
async fn index_css() -> impl Responder {
    include_str!("../../http-panel/dist/index.css")
        .customize().insert_header(("Content-Type", "text/css"))
}

#[get("/index.js")]
async fn index_js() -> impl Responder {
    include_str!("../../http-panel/dist/index.js")
        .customize().insert_header(("Content-Type", "application/javascript"))
}

#[post("api")]
async fn api(cmds: Json<Vec<handle::Request>>) -> impl Responder {
    let mut responses: Vec<handle::Response> = Vec::new();
    for request in cmds.0 {
        match command::handle_exec(request).await {
            Ok(response) => {
                responses.push(response);
            }
            Err(e) => {
                let response = handle::Response::failed(e.to_string());
                responses.push(response);
            }
        }
    }
    let body = serde_json::to_vec(&responses).unwrap();
    body
}

#[derive(Deserialize)]
struct QueryParams {
    pid: Option<usize>,
}

#[get("/api/matrix")]
async fn matrix(param: web::Query<QueryParams>) -> impl Responder {
    info!("收到matrix请求 pid:{}", param.pid.unwrap());
    // let body = serde_json::to_vec(&command::matrix(param.pid).unwrap()).unwrap();
    match param.pid {
        Some(pid) => match command::matrix(pid) {
            Ok(matrix) => HttpResponse::Ok().json(matrix),  // 成功时返回 JSON 数据
            Err(e) => {
                info!("error in matrix, pid: {:?}", param.pid);
                HttpResponse::InternalServerError().body(format!("Error occurred: {}", e))
            }
        },
        None => HttpResponse::BadRequest().body("Missing 'pid' parameter"),
    }
}



