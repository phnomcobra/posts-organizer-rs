use chrono::Utc;
use log::LevelFilter;
use std::{io::Write, path::Path};

pub fn init_logger(app_loglevel: LevelFilter, api_loglevel: LevelFilter) {
    env_logger::builder()
        .filter_level(app_loglevel)
        .filter_module("actix_web", api_loglevel)
        .filter_module("actix_server", api_loglevel)
        .filter_module("actix_http", api_loglevel)
        .filter_module("tracing_actix_web", api_loglevel)
        .format(move |buf, record| {
            let short_filename = record
                .file()
                .and_then(|f| Path::new(f).file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            writeln!(
                buf,
                "{}|{}|{}:L{}|{}|{}",
                record.level(),
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                short_filename,
                record.line().unwrap_or(0),
                record.target(),
                record.args()
            )
        })
        .parse_default_env()
        .init();
}
