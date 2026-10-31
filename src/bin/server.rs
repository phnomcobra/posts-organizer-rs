use actix_web::{rt::spawn, web, App, HttpServer, Result};
use chrono::Utc;
use clokwerk::{AsyncScheduler, Interval::Seconds};
use tracing_actix_web::TracingLogger;

use std::time::Duration;
use tokio::time::sleep;

use posts_organizer_rs::{
    collections::vacuum_collections,
    logger::init_logger,
    models::config::Config,
    processor::{catch_all_get_handler, catch_all_post_handler, expire_posts},
};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }

    let config = Config::load();

    init_logger(&config.log_level_app.to_string(), &config.log_level_api.to_string());

    config.log();

    let mut scheduler = AsyncScheduler::new();

    scheduler.every(Seconds(60)).run(|| async {
        let config = Config::load();
        expire_posts(Utc::now().timestamp() as u32 - config.timeout_secs)
            .unwrap_or_else(|e| log::error!("Error expiring posts: {e}"));
    });

    scheduler.every(Seconds(600)).run(|| async {
        vacuum_collections()
            .unwrap_or_else(|e| log::error!("Error vacuuming collections: {e}"));
    });

    log::info!("Starting scheduler");

    spawn({
        async move {
            loop {
                scheduler.run_pending().await;
                sleep(Duration::from_secs(1)).await;
            }
        }
    });

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::PayloadConfig::new(usize::MAX))
            .route("/{tail:.*}", web::get().to(catch_all_get_handler))
            .route("/{tail:.*}", web::post().to(catch_all_post_handler))
    })
    .workers(config.workers);

    log::info!("Starting server");

    server
        .bind((config.socket_host.as_str(), config.socket_port))?
        .run()
        .await
}
