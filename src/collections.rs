//! Shared collection singletons for posts and key-value storage.
//!
//! Each public opener returns a cheap clone of a process-wide singleton
//! `Collection<T>`.  Because `Collection` wraps an `Arc<Mutex<Connection>>`
//! internally, all clones share the same SQLite connection and therefore the
//! same underlying data — whether the database is file-backed or in-memory.
//!
//! In-memory collections use a `file:?mode=memory` URI so that a single
//! named connection is created once and reused for the lifetime of the
//! process.  File-backed collections (keys, peers, kvstore) open their usual
//! `{name}.sqlite` path.

use std::sync::OnceLock;

use anyhow::Result;

use crate::dao::collection::Collection;
use crate::dao::db_path;
use crate::dao::kvstore::KeyValuePair;
use crate::models::posts::Post;

// ── Singletons ────────────────────────────────────────────────────────────────

static POSTS:   OnceLock<Collection<Post>>         = OnceLock::new();
static KVSTORE: OnceLock<Collection<KeyValuePair>> = OnceLock::new();

// ── Collection openers ────────────────────────────────────────────────────────

/// Open (or return the cached singleton for) the `posts` collection.
pub fn open_posts() -> Result<Collection<Post>> {
    Ok(POSTS.get_or_init(|| {
        let c = Collection::new("posts", Some(db_path("posts").as_ref()))
            .expect("failed to open posts collection");
        c.create_attribute("path", "/path").ok();
        c.create_attribute("create_time", "/create_time").ok();
        c
    }).clone())
}

/// Open (or return the cached singleton for) the `kvstore` collection.
pub fn open_kvstore() -> Result<Collection<KeyValuePair>> {
    Ok(KVSTORE.get_or_init(|| {
        let c = Collection::new("kvstore", Some(db_path("kvstore").as_ref()))
            .expect("failed to open kvstore collection");
        c.create_attribute("name", "/name").ok();
        c
    }).clone())
}

/// Vacuum all collections to reclaim space from deleted records.
pub fn vacuum_collections() -> Result<()> {
    open_posts()?.document.vacuum()?;
    open_kvstore()?.document.vacuum()?;
    Ok(())
}
