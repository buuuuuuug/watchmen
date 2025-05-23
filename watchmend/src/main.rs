use colored::Colorize;
use std::{error::Error, path::Path};
use log::error;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};
use watchmend::common::{arg::DaemonArgs, config::Config};
use watchmend::{engine, monitor::run_monitor};
use watchmend::scheduled_task::run_scheduled;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let clargs = DaemonArgs::new();
    if clargs.version {
        println!(
            "{} {}",
            "Watchmen rust".green(),
            env!("CARGO_PKG_VERSION").green()
        );
        return Ok(());
    }

    let load = clargs.load;

    let config: Config = Config::init(clargs.config.clone())?;

    // ====================================================

    let log_path = config
        .watchmen
        .log_dir
        .clone()
        .unwrap_or("./logs".to_string());
    let path: &Path = Path::new(&log_path);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    let mut filter = EnvFilter::from_default_env();
    if Some("debug".to_string()) == config.watchmen.log_level {
        filter = filter.add_directive(Level::ERROR.into());
        filter = filter.add_directive(Level::WARN.into());
        filter = filter.add_directive(Level::INFO.into());
        filter = filter.add_directive(Level::DEBUG.into());
    } else if Some("info".to_string()) == config.watchmen.log_level {
        filter = filter.add_directive(Level::ERROR.into());
        filter = filter.add_directive(Level::WARN.into());
        filter = filter.add_directive(Level::INFO.into());
    } else if Some("warn".to_string()) == config.watchmen.log_level {
        filter = filter.add_directive(Level::ERROR.into());
        filter = filter.add_directive(Level::WARN.into());
    } else if Some("error".to_string()) == config.watchmen.log_level {
        filter = filter.add_directive(Level::ERROR.into());
    } else {
        filter = filter.add_directive(Level::ERROR.into());
        filter = filter.add_directive(Level::WARN.into());
        filter = filter.add_directive(Level::INFO.into());
    }
    let appender = tracing_appender::rolling::daily("./logs", "watchmen.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = fmt::Subscriber::builder()
        .with_writer(non_blocking_appender)
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to set tracing default subscriber");

    // ====================================================

    info!(
        "Watchmen daemon start by engines {:?}",
        config.watchmen.engines
    );

    tokio::spawn(async move {
        let _ = run_monitor(config.watchmen.interval).await;
    });
    tokio::spawn(async move {
        let _ = run_scheduled(config.watchmen.schedule_interval).await;
    });
    if let Err(e) = engine::start(config, load).await {
        error!("http error start error: {}", e);
    }
    Ok(())
}
