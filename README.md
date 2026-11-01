# posts-organizer-rs

A lightweight Rust HTTP service that accepts arbitrary `POST` bodies at any
URL path and lets them be retrieved later with a `GET` to that same path. It
is backed by a small, schema-free, SQLite-based document/collection storage
layer, and ships with a set of companion CLI tools for configuration, load
testing, and reporting.

## Features

- **Catch-all HTTP endpoint** — `POST` to any path stores the raw request
  body; `GET` on that same path retrieves (and removes) the most recently
  stored post for it.
- **Automatic expiration & vacuuming** — a background scheduler periodically
  expires posts older than a configurable timeout and reclaims disk space.
- **Centralized runtime configuration** — all settings live in a small
  SQLite-backed key-value store, managed with the `posts-configure` CLI
  (no config files or environment variables required, though both are
  supported).
- **Companion tooling**:
  - `posts-bench` — a load-testing tool that drives increasing
    concurrency/payload-size traffic against the server and reports
    human-readable bandwidth and latency.
  - `posts-stat` — a reporting tool that aggregates stored posts by URL
    path (count, total size, most recent post age).
- **Debian packaging** — a build script and `systemd` unit for deploying the
  server as a managed service.

## Binaries

| Binary            | Description                                       |
|-------------------|---------------------------------------------------|
| `posts-server`    | The HTTP server.                                  |
| `posts-configure` | CLI to view/set runtime configuration.            |
| `posts-bench`     | Load-testing tool for a running `posts-server`.   |
| `posts-stat`      | Reports per-path statistics for the stored posts. |

## Getting started

### Prerequisites

- Rust (version pinned in [rust-toolchain](rust-toolchain)); install via
  [rustup](https://rustup.rs/).

### Build

```sh
cargo build --release
```

Binaries are written to `target/release/`.

### Run

```sh
# Optional: review or change settings before starting the server.
./target/release/posts-configure --view

# Start the server (binds 0.0.0.0:8080 by default).
./target/release/posts-server
```

By default, SQLite database files (`posts.sqlite`, `kvstore.sqlite`) are
created in the current working directory. Building with the `debian`
feature (see [Debian packaging](#debian-packaging)) places them in
`/var/posts-organizer-rs/` instead.

### Try it out

```sh
curl -X POST --data "hello world" http://localhost:8080/notes/1
curl http://localhost:8080/notes/1
# -> hello world
```

## Configuration

Settings are stored in the `kvstore` SQLite collection and read once per
process at startup. Use `posts-configure` to view or change them:

```sh
posts-configure --view
posts-configure --workers 4 --port 9090 --log-level-api WARNING
```

| Setting         | Flag                 | Default   | Description                                                          |
|-----------------|----------------------|-----------|----------------------------------------------------------------------|
| `workers`       | `-w`, `--workers`    | `2`       | Number of HTTP worker threads/processes.                             |
| `socket_host`   | `-d`, `--host`       | `0.0.0.0` | Address the server binds to.                                         |
| `socket_port`   | `-p`, `--port`       | `8080`    | Port the server binds to.                                            |
| `log_level_app` | `--log-level-app`    | `INFO`    | Application log level (`DEBUG`/`INFO`/`WARNING`/`ERROR`/`CRITICAL`). |
| `log_level_api` | `--log-level-api`    | `INFO`    | Log level for the Actix Web/API framework.                           |
| `timeout_secs`  | `--timeout-secs`     | `86400`   | Seconds a post is kept before being expired.                         |

`posts-configure` also supports `-e`/`--load-env`, which loads settings from
`POSTS_WORKERS`, `POSTS_HOST`, `POSTS_PORT`, `POSTS_LOG_LEVEL_APP`,
`POSTS_LOG_LEVEL_API`, and `POSTS_TIMEOUT_SECS` environment variables.

## Load testing with `posts-bench`

```sh
posts-bench
```

Reads connection settings from the same configuration as the server, then
sends `POST` and `GET` traffic to `/benchmark` across every combination of
concurrency (1 through 64, doubling) and payload size (1 KiB through 8 MiB,
doubling), printing a table of bandwidth and latency per combination.

## Reporting with `posts-stat`

```sh
posts-stat
```

Reads the `posts` collection's indexed attributes directly (without
decoding post bodies) and prints a table of post count, combined size, and
most recent post age, grouped by URL path.

## Architecture

- [`src/dao/`](src/dao) — a small, generic, SQLite-backed storage layer:
  - `Document` — low-level schema management (collections, indexed
    attributes, objects) over a single SQLite connection.
  - `Collection<T>` — a typed view over a `Document`'s objects, supporting
    attribute-based queries (`$eq`, `$gt`, `$lt`, `$contains`, `$regex`, …).
  - `Object<T>` — a single stored record.
  - `KVStore` — a simple key-value collection used for runtime
    configuration.
  - `Datastore`/`File` — chunked binary storage built on top of a
    `Collection`.
- [`src/models/`](src/models) — data types stored in collections (`Post`,
  `Config`).
- [`src/collections.rs`](src/collections.rs) — process-wide singleton
  openers for the `posts` and `kvstore` collections.
- [`src/processor.rs`](src/processor.rs) — the catch-all HTTP handlers and
  post-expiration logic.
- [`src/format.rs`](src/format.rs) — shared human-readable formatting
  helpers (byte sizes, ages) used by the CLI tools.
- [`src/bin/`](src/bin) — the four binaries described above.

## Development

```sh
cargo build                # build the library and all binaries
cargo test                 # run unit tests
cargo clippy --all-targets # lint
```

## Debian packaging

```sh
./scripts/build-deb.sh
```

Builds release binaries with the `debian` feature enabled (which changes
the SQLite database directory to `/var/posts-organizer-rs/`) and produces a
`.deb` package in `dist/` containing all four binaries, a `systemd` unit
([`etc/posts-server.service`](etc/posts-server.service)), and `postinst`/
`prerm` scripts that enable and start/stop the `posts-server` service.

## License

[MIT](LICENSE)
