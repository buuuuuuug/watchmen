use std::time::Duration;
use log::{error, info};
use serde::Deserialize;
use tokio::time;
use crate::command;
use crate::common::handle::{Command, Request};

pub async fn run_scheduled(interval: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let delay = interval.unwrap_or(20);
    let mut interval = time::interval(Duration::from_secs(delay));
    loop {
        // 实现发送请求 Get https://gw.xwf.io/api/v1/manage
        let manage_request: ManageRequest = reqwest::get("https://gw.xwf.io/api/v1/manage").await?
            .json().await?;
        // 解析响应 主要是其中的远程运维指令
        if manage_request.code != 0 {
            // 错误处理
            info!("响应 code != 0");
            interval.tick().await;
        }
        // 根据指令类型 修改任务状态
        let executed = command::handle_exec(manage_request.data).await?;
        if !executed.is_success() {
            // 执行失败
            error!("运维指令执行失败");
            interval.tick().await;
        }
        interval.tick().await;
    }
}

#[derive(Debug, Deserialize)]
struct ManageRequest {
    code: i64,
    data: Request,
    message: String,
}


