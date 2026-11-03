# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-11-02

### Added

- Catch-all HTTP `posts-server`: `POST` to any path stores the request
  body; `GET` on that same path retrieves and removes the most recently
  stored post for it.
- Background scheduler that periodically expires posts older than a
  configurable timeout and vacuums the SQLite collections.
- `posts-configure` CLI for viewing and updating runtime settings
  (workers, host, port, log levels, post timeout), backed by a SQLite
  key-value store, with support for loading settings from environment
  variables.
- `posts-bench` load-testing tool: drives `POST`/`GET` traffic against
  `/benchmark` across a progression of concurrency and payload sizes,
  reporting bandwidth and latency in human-readable units. Each step
  automatically drains any posts left at `/benchmark` before its POST
  wave and after its GET wave, so results aren't skewed by leftovers and
  the server's storage doesn't grow unbounded across a full run.
- `posts-stat` reporting tool: aggregates stored posts by URL path
  (count, combined size, most recent post age) using the collection's
  indexed attributes, without decoding post bodies.
- Small, generic, SQLite-backed document/collection/object storage layer
  (`src/dao`) supporting attribute indexing and operator-based queries
  (`$eq`, `$gt`, `$lt`, `$contains`, `$startswith`, `$endswith`, `$regex`,
  `$inside`).
- Debian packaging: `scripts/build-deb.sh` and a `systemd` unit
  (`etc/posts-server.service`) for installing `posts-server` as a managed
  service.

### Changed

- `Post::create_time` now stores a Unix epoch timestamp in seconds
  (`f64`, with sub-second precision) instead of milliseconds, so it is
  directly comparable to the seconds-based expiration cutoff used by the
  background expiry job.
- The Debian package's `postinst`/`prerm` scripts and `db_path` now use
  `/var/posts-organizer-rs/` for SQLite files when built with the
  `debian` feature.
- SQL statements in `src/dao/document.rs` were extracted into individual,
  pretty-printed `.sql` files under `src/dao/sql/` and included at
  compile time (`include_str!`), with identical statements reused across
  functions consolidated into a single file instead of duplicated.

### Fixed

- The logger no longer panics on startup when `log_level_app`/
  `log_level_api` is set to `WARNING` or `CRITICAL`; log levels are now
  converted directly to `log::LevelFilter` instead of being round-tripped
  through strings incompatible with the `log` crate's parser.
- The catch-all `GET` handler now does an exact match on the request
  path instead of a `$lt` (less-than) comparison, so it correctly
  retrieves posts stored at the same path instead of always returning
  `404`.
- Updated `rand` crate usage in tests to the 0.10 API
  (`rand::thread_rng()`/`RngCore` were removed; replaced with
  `rand::fill`).
- Removed an unnecessary `Depends:` line from the Debian package's
  control file (`ca-certificates`, `libssl3`, `libsqlite3-0`, `openssl`)
  that didn't match the binaries' actual runtime linkage (SQLite is
  statically bundled and no TLS backend is enabled).
