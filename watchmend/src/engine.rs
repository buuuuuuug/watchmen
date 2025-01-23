use crate::common::config::Config;
use actix_web::{App, HttpServer};
use log::error;
use tokio::sync::mpsc;
use tracing::info;

use crate::global;

#[cfg(feature = "sock")]
pub mod sock;

#[cfg(feature = "socket")]
pub mod socket;

#[cfg(feature = "http")]
mod actix_http;


pub async fn start(config: Config, load: bool) -> Result<(), Box<dyn std::error::Error>> {
    if load {
        if let Some(path) = config.watchmen.cache.clone() {
            global::set_cache(path.clone()).await;
            match global::load(&path).await {
                Ok(_) => {
                    info!("Cache tasks loaded.");
                    println!("Cache tasks loaded.");
                }
                Err(e) => {
                    info!("Cache tasks load failed: {}", e);
                    println!("Cache tasks load failed: {}", e);
                    error!("load config error");
                }
            }
        } else {
            info!("Cache tasks load failed: cache file not set in config.");
            println!("Cache tasks load failed: cache file not set in config.");
        }
    }

    let (tx, mut rx) = mpsc::channel::<i32>(12);

    let tx_ctrl_c = tx.clone(); // 监听到 ctrl c 通信管道
    let tx_ctrl_d = tx.clone(); // 监听到 ctrl d 通信管道

    // ctrl c 停止运行 / terminate on ctrl-c
    let s_ctrl_c = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        tx_ctrl_c.send(9).await.unwrap();
    });

    // ctrl d 停止运行 / terminate on ctrl-d
    let s_ctrl_d = tokio::spawn(async move {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
        tx_ctrl_d.send(15).await.unwrap();
    });

    #[cfg(feature = "sock")]
    // sock in config.watchmen.engines ?
    let joinhandle_sock = if config.watchmen.engines.contains(&"sock".to_string()) {
        info!("Starting sock...");
        println!("Starting sock...");
        Some(sock::start(config.clone()).await)
    } else {
        None
    };

    #[cfg(feature = "socket")]
    let joinhandle_socket = if config.watchmen.engines.contains(&"socket".to_string()) {
        info!("Starting socket...");
        println!("Starting socket...");
        Some(socket::start(config.clone()).await)
    } else {
        None
    };

    #[cfg(feature = "http")]
    if config.watchmen.engines.contains(&"http".to_string()) {
        info!("Starting http...");
        println!("Starting http...");
        // Some(http::start(config.clone()).await)
        HttpServer::new(|| {
            App::new().service(actix_http::index)
                .service(actix_http::index_html)
                .service(actix_http::favicon)
                .service(actix_http::index_css)
                .service(actix_http::index_js)
                .service(actix_http::api)
                .service(actix_http::matrix)
        })
        .bind((config.http.host, config.http.port))?
        .run()
        .await?;
    } else {
        return Err(Box::from("No engines started."));
    };

    info!("All engines started.");
    println!("All engines started.");

    // ================== Wait for all tasks to complete ==================

    let _res = rx.recv().await;

    s_ctrl_c.abort();
    s_ctrl_d.abort();

    println!("Shutting down...");
    #[cfg(feature = "sock")]
    if config.watchmen.engines.contains(&"sock".to_string()) && joinhandle_sock.is_some() {
        joinhandle_sock.unwrap().abort();
    }

    #[cfg(feature = "socket")]
    if config.watchmen.engines.contains(&"socket".to_string()) && joinhandle_socket.is_some() {
        joinhandle_socket.unwrap().abort();
    }
    Ok(())
}
