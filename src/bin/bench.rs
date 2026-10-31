//! Load-testing tool for `posts-server`.
//!
//! Connection settings (host/port) are read from the same kvstore-backed
//! `Config` used by the server, so run `posts-configure` first to point
//! this tool at the right instance.
//!
//! The tool drives a progression of increasing load against the
//! `/benchmark` path: concurrency and payload size each double, from
//! concurrency 1 / 1 KiB up to concurrency 64 / 8 MiB, and every
//! concurrency level is tested against every payload size. Both POST
//! (upload) and GET (download) traffic are measured at every step, and
//! bandwidth/latency are reported in human-readable units.

use std::time::{Duration, Instant};

use bytes::Bytes;
use reqwest::{Client, Method};
use tokio::task::JoinSet;

use posts_organizer_rs::models::config::Config;

/// Largest concurrency level tested.
const MAX_CONCURRENCY: usize = 64;
/// Smallest payload tested.
const MIN_PAYLOAD: usize = 1024; // 1 KiB
/// Largest payload tested.
const MAX_PAYLOAD: usize = 8 * 1024 * 1024; // 8 MiB

/// One step of the load progression: how many requests to fire concurrently
/// and how large each request body should be.
struct Step {
    concurrency: usize,
    payload_size: usize,
}

/// Build the progression: every concurrency level (1, 2, 4, ..., up to
/// `MAX_CONCURRENCY`) is tested against every payload size (1 KiB, 2 KiB,
/// ..., up to `MAX_PAYLOAD`), producing the full cross product of steps.
fn progression() -> Vec<Step> {
    let mut concurrency_levels = Vec::new();
    let mut concurrency = 1usize;
    while concurrency <= MAX_CONCURRENCY {
        concurrency_levels.push(concurrency);
        concurrency *= 2;
    }

    let mut payload_sizes = Vec::new();
    let mut payload_size = MIN_PAYLOAD;
    while payload_size <= MAX_PAYLOAD {
        payload_sizes.push(payload_size);
        payload_size *= 2;
    }

    let mut steps = Vec::with_capacity(concurrency_levels.len() * payload_sizes.len());
    for &payload_size in &payload_sizes {
        for &concurrency in &concurrency_levels {
            steps.push(Step {
                concurrency,
                payload_size,
            });
        }
    }
    steps
}

/// Result of firing one wave of `concurrency` concurrent requests.
struct WaveResult {
    /// Total request + response bytes across all successful requests.
    bytes_transferred: u64,
    /// Wall-clock time for the whole wave to complete.
    elapsed: Duration,
    /// Per-request round-trip latency, one entry per successful request.
    latencies: Vec<Duration>,
    /// Number of requests that failed (connection errors, timeouts, etc.).
    errors: usize,
}

/// Fire `concurrency` concurrent requests of `method` at `url`, optionally
/// sending `body` on each request, and measure the resulting throughput and
/// per-request latency.
async fn run_wave(client: &Client, method: Method, url: &str, concurrency: usize, body: Option<Bytes>) -> WaveResult {
    let start = Instant::now();
    let mut tasks = JoinSet::new();

    for _ in 0..concurrency {
        let client = client.clone();
        let url = url.to_string();
        let method = method.clone();
        let body = body.clone();
        tasks.spawn(async move {
            let request_start = Instant::now();
            let mut builder = client.request(method, url);
            let request_len = match &body {
                Some(b) => {
                    builder = builder.body(b.clone());
                    b.len()
                }
                None => 0,
            };
            let response = builder.send().await.ok()?;
            let response_len = response.bytes().await.ok()?.len();
            Some((request_len as u64 + response_len as u64, request_start.elapsed()))
        });
    }

    let mut bytes_transferred = 0u64;
    let mut latencies = Vec::with_capacity(concurrency);
    let mut errors = 0usize;
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Some((bytes, latency))) => {
                bytes_transferred += bytes;
                latencies.push(latency);
            }
            _ => errors += 1,
        }
    }

    WaveResult {
        bytes_transferred,
        elapsed: start.elapsed(),
        latencies,
        errors,
    }
}

/// Fill a buffer of `size` bytes with random data to use as a request body.
fn random_payload(size: usize) -> Bytes {
    let mut buf = vec![0u8; size];
    rand::fill(&mut buf[..]);
    Bytes::from(buf)
}

/// Format a byte count using the appropriate binary unit (B/KB/MB/GB).
fn format_bytes(bytes: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.2} {}", UNITS[unit])
}

/// Format a transfer rate (bytes/sec) using the appropriate binary unit.
fn format_rate(bytes_per_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

/// Format a duration using the appropriate unit (µs/ms/s).
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs_f64();
    if secs < 1e-3 {
        format!("{:.2} µs", secs * 1_000_000.0)
    } else if secs < 1.0 {
        format!("{:.2} ms", secs * 1_000.0)
    } else {
        format!("{secs:.2} s")
    }
}

const COLUMNS: (usize, usize, usize, usize, usize) = (6, 11, 12, 14, 12);

fn print_header() {
    let (c0, c1, c2, c3, c4) = COLUMNS;
    println!(
        "{:<c0$} | {:>c1$} | {:>c2$} | {:>c3$} | {:>c4$}",
        "Method", "Concurrency", "Payload", "Bandwidth", "Latency"
    );
    println!("{}", "-".repeat(c0 + c1 + c2 + c3 + c4 + 12));
}

fn print_row(method: &str, step: &Step, result: &WaveResult) {
    let (c0, c1, c2, c3, c4) = COLUMNS;
    let payload = format_bytes(step.payload_size as f64);

    let (bandwidth, latency) = if result.latencies.is_empty() {
        ("N/A".to_string(), "N/A".to_string())
    } else {
        let bandwidth = result.bytes_transferred as f64 / result.elapsed.as_secs_f64();
        let mean_latency = result.latencies.iter().sum::<Duration>() / result.latencies.len() as u32;
        (format_rate(bandwidth), format_duration(mean_latency))
    };

    println!(
        "{:<c0$} | {:>c1$} | {:>c2$} | {:>c3$} | {:>c4$}",
        method, step.concurrency, payload, bandwidth, latency
    );

    if result.errors > 0 {
        eprintln!(
            "  warning: {} of {} {method} request(s) failed at concurrency={}, payload={payload}",
            result.errors,
            result.errors + result.latencies.len(),
            step.concurrency
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();
    // 0.0.0.0 is a valid bind address but not a valid address to connect to.
    let host = if config.socket_host == "0.0.0.0" {
        "127.0.0.1".to_string()
    } else {
        config.socket_host.clone()
    };
    let url = format!("http://{host}:{}/benchmark", config.socket_port);

    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    if let Err(error) = client.get(&url).send().await {
        eprintln!("Error: failed to reach {url}: {error}");
        eprintln!("Make sure posts-server is running and reachable at that address (use posts-configure to adjust host/port).");
        std::process::exit(1);
    }

    eprintln!("Benchmarking {url}\n");

    print_header();
    for step in progression() {
        let payload = random_payload(step.payload_size);

        let post_result = run_wave(&client, Method::POST, &url, step.concurrency, Some(payload)).await;
        print_row("POST", &step, &post_result);

        let get_result = run_wave(&client, Method::GET, &url, step.concurrency, None).await;
        print_row("GET", &step, &get_result);
    }

    Ok(())
}
