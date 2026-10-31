use clap::Parser;
use posts_organizer_rs::{
    dao::kvstore::KVStore,
    models::config::{Config, LogLevel},
};

#[derive(Parser, Debug)]
#[clap(name = "posts-configure", about = "Configure posts organizer settings")]
struct Args {
    #[clap(short = 'w', long, help = "Number of worker processes")]
    workers: Option<usize>,

    #[clap(short = 'd', long, help = "Server host address")]
    host: Option<String>,

    #[clap(short = 'p', long, help = "Server TCP port")]
    port: Option<u16>,

    #[clap(
        long,
        help = "Application log level (DEBUG/INFO/WARNING/ERROR/CRITICAL)"
    )]
    log_level_app: Option<String>,

    #[clap(
        long,
        help = "API/framework log level (DEBUG/INFO/WARNING/ERROR/CRITICAL)"
    )]
    log_level_api: Option<String>,

    #[clap(long, help = "Seconds before a post is discarded")]
    timeout_secs: Option<u32>,

    #[clap(short = 'v', long, help = "View current configuration settings")]
    view: bool,

    #[clap(
        short = 'e',
        long = "load-env",
        help = "Load configuration from environment variables"
    )]
    load_env: bool,
}

fn load_from_environment(store: &KVStore) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(v) = std::env::var("POSTS_WORKERS") {
        if let Ok(w) = v.parse::<usize>() {
            store.commit("workers", w as u64)?;
            println!("✓ Loaded POSTS_WORKERS: {v}");
        }
    }
    if let Ok(v) = std::env::var("POSTS_HOST") {
        store.commit("socket_host", v.as_str())?;
        println!("✓ Loaded POSTS_HOST: {v}");
    }
    if let Ok(v) = std::env::var("POSTS_PORT") {
        if let Ok(port) = v.parse::<u16>() {
            store.commit("socket_port", port)?;
            println!("✓ Loaded POSTS_PORT: {v}");
        }
    }
    if let Ok(v) = std::env::var("POSTS_LOG_LEVEL_APP") {
        if v.to_uppercase().parse::<LogLevel>().is_ok() {
            store.commit("log_level_app", v.to_uppercase())?;
            println!("✓ Loaded POSTS_LOG_LEVEL_APP: {v}");
        }
    }
    if let Ok(v) = std::env::var("POSTS_LOG_LEVEL_API") {
        if v.to_uppercase().parse::<LogLevel>().is_ok() {
            store.commit("log_level_api", v.to_uppercase())?;
            println!("✓ Loaded POSTS_LOG_LEVEL_API: {v}");
        }
    }
    if let Ok(v) = std::env::var("POSTS_TIMEOUT_SECS") {
        if let Ok(n) = v.parse::<u32>() {
            store.commit("timeout_secs", n)?;
            println!("✓ Loaded POSTS_TIMEOUT_SECS: {v}");
        }
    }
    Ok(())
}

fn display_config() {
    let config = Config::load();
    println!("\n{}", "=".repeat(50));
    println!("Current Configuration");
    println!("{}", "=".repeat(50));
    let items = [
        ("Workers", config.workers.to_string()),
        ("Socket Host", config.socket_host.clone()),
        ("Socket Port", config.socket_port.to_string()),
        ("Log Level App", config.log_level_app.to_string()),
        ("Log Level API", config.log_level_api.to_string()),
        ("Timeout Secs", config.timeout_secs.to_string()),
    ];
    for (label, value) in &items {
        let display = if value.len() > 60 {
            format!("{}...", &value[..60])
        } else {
            value.clone()
        };
        println!("{label:.<25} {display}");
    }
    println!("{}", "=".repeat(50));
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw: Vec<String> = std::env::args()
        .map(|a| if a == "-?" { "--help".to_string() } else { a })
        .collect();
    let args = Args::parse_from(raw);
    let store = KVStore::new(None).expect("failed to open kvstore");
    let mut modified = false;

    if args.load_env {
        println!("Loading configuration from environment variables...");
        load_from_environment(&store)?;
        println!("✓ Successfully loaded configuration from environment variables");
        modified = true;
    }

    if let Some(v) = args.workers {
        store.commit("workers", v as u64)?;
        println!("✓ Set Workers: {v}");
        modified = true;
    }
    if let Some(v) = args.host {
        store.commit("socket_host", v.as_str())?;
        println!("✓ Set Host: {v}");
        modified = true;
    }
    if let Some(v) = args.port {
        store.commit("socket_port", v)?;
        println!("✓ Set Port: {v}");
        modified = true;
    }
    if let Some(v) = args.log_level_app {
        let upper = v.to_uppercase();
        if upper.parse::<LogLevel>().is_ok() {
            store.commit("log_level_app", upper.as_str())?;
            println!("✓ Set Log Level App: {upper}");
            modified = true;
        } else {
            eprintln!("Error: invalid log level '{v}'");
        }
    }
    if let Some(v) = args.log_level_api {
        let upper = v.to_uppercase();
        if upper.parse::<LogLevel>().is_ok() {
            store.commit("log_level_api", upper.as_str())?;
            println!("✓ Set Log Level API: {upper}");
            modified = true;
        } else {
            eprintln!("Error: invalid log level '{v}'");
        }
    }
    if let Some(v) = args.timeout_secs {
        store.commit("timeout_secs", v)?;
        println!("✓ Set Timeout Secs: {v}");
        modified = true;
    }

    if args.view {
        display_config();
    } else if !modified {
        println!("No options provided. Use --help for usage information.");
    }

    Ok(())
}
