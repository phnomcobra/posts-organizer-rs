use actix_web::{rt::spawn, web, App, HttpServer, Result};
use clokwerk::{AsyncScheduler, Interval::Seconds};
use tracing_actix_web::TracingLogger;

use std::time::Duration;
use tokio::time::sleep;

use posts_organizer_rs::{
    logger::init_logger,
};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "1");
        }
    }

    init_logger(
        "debug",
        "debug",
    );

    let mut scheduler = AsyncScheduler::new();

    // scheduler.every(Seconds(60)).run(|| async {
    //    vacuum_collections().unwrap_or_else(|e| log::error!("Error vacuuming collections: {e}"));
    // });

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
    })
    .workers(2);

    log::info!("Starting server");

    server
        .bind(("0.0.0.0", 8081u16))?
        .run()
        .await
}
