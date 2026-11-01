//! Statistics tool for the `posts` collection.
//!
//! Uses `Collection::list_index_entries` against the indexed `path`, `size`,
//! and `create_time` attributes to aggregate per-path statistics without
//! decoding any post bodies, then prints a summary table.

use std::collections::HashMap;

use chrono::Utc;

use posts_organizer_rs::collections::open_posts;
use posts_organizer_rs::format::{format_age, format_bytes};

/// Aggregate statistics for all posts stored under a single URL path.
#[derive(Default)]
struct PathStats {
    count: usize,
    total_size: u64,
    latest_create_time: f64,
}

fn main() -> anyhow::Result<()> {
    let posts = open_posts()?;

    let paths = posts.list_index_entries("path")?;
    let sizes: HashMap<String, String> = posts.list_index_entries("size")?.into_iter().collect();
    let create_times: HashMap<String, String> =
        posts.list_index_entries("create_time")?.into_iter().collect();

    if paths.is_empty() {
        println!("No posts found.");
        return Ok(());
    }

    let rows = aggregate(paths, &sizes, &create_times);
    print_table(&rows, Utc::now().timestamp_millis() as f64 / 1000.0);

    Ok(())
}

/// Group `(objuuid, path)` index entries by path, summing post counts and
/// sizes and tracking the most recent `create_time` (epoch seconds) per
/// path. Missing or unparseable `size`/`create_time` entries default to
/// zero. Rows are sorted alphabetically by path.
fn aggregate(
    paths: Vec<(String, String)>,
    sizes: &HashMap<String, String>,
    create_times: &HashMap<String, String>,
) -> Vec<(String, PathStats)> {
    let mut stats: HashMap<String, PathStats> = HashMap::new();
    for (objuuid, path) in paths {
        let size: u64 = sizes.get(&objuuid).and_then(|s| s.parse().ok()).unwrap_or(0);
        let create_time: f64 = create_times
            .get(&objuuid)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let entry = stats.entry(path).or_default();
        entry.count += 1;
        entry.total_size += size;
        entry.latest_create_time = entry.latest_create_time.max(create_time);
    }

    let mut rows: Vec<(String, PathStats)> = stats.into_iter().collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

fn print_table(rows: &[(String, PathStats)], now_secs: f64) {
    let path_width = rows
        .iter()
        .map(|(path, _)| path.chars().count())
        .max()
        .unwrap_or(0)
        .max("Path".len());
    const COUNT_WIDTH: usize = 5;
    const SIZE_WIDTH: usize = 12;
    const AGE_WIDTH: usize = 14;

    println!(
        "{:<path_width$} | {:>COUNT_WIDTH$} | {:>SIZE_WIDTH$} | {:>AGE_WIDTH$}",
        "Path", "Posts", "Size", "Latest Post"
    );
    println!(
        "{}",
        "-".repeat(path_width + COUNT_WIDTH + SIZE_WIDTH + AGE_WIDTH + 9)
    );

    for (path, stats) in rows {
        let age_secs = (now_secs - stats.latest_create_time) as i64;
        println!(
            "{:<path_width$} | {:>COUNT_WIDTH$} | {:>SIZE_WIDTH$} | {:>AGE_WIDTH$}",
            path,
            stats.count,
            format_bytes(stats.total_size as f64),
            format_age(age_secs),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_groups_by_path_and_sums_size() {
        let paths = vec![
            ("a".to_string(), "/foo".to_string()),
            ("b".to_string(), "/foo".to_string()),
            ("c".to_string(), "/bar".to_string()),
        ];
        let sizes: HashMap<String, String> = [
            ("a".to_string(), "100".to_string()),
            ("b".to_string(), "50".to_string()),
            ("c".to_string(), "10".to_string()),
        ]
        .into_iter()
        .collect();
        let create_times: HashMap<String, String> = [
            ("a".to_string(), "1000.5".to_string()),
            ("b".to_string(), "2000.25".to_string()),
            ("c".to_string(), "500.1".to_string()),
        ]
        .into_iter()
        .collect();

        let rows = aggregate(paths, &sizes, &create_times);
        assert_eq!(rows.len(), 2);

        // Sorted alphabetically: "/bar" < "/foo".
        let (bar_path, bar_stats) = &rows[0];
        assert_eq!(bar_path, "/bar");
        assert_eq!(bar_stats.count, 1);
        assert_eq!(bar_stats.total_size, 10);
        assert_eq!(bar_stats.latest_create_time, 500.1);

        let (foo_path, foo_stats) = &rows[1];
        assert_eq!(foo_path, "/foo");
        assert_eq!(foo_stats.count, 2);
        assert_eq!(foo_stats.total_size, 150);
        assert_eq!(foo_stats.latest_create_time, 2000.25);
    }

    #[test]
    fn test_aggregate_missing_size_or_create_time_defaults_to_zero() {
        let paths = vec![("a".to_string(), "/foo".to_string())];
        let rows = aggregate(paths, &HashMap::new(), &HashMap::new());
        assert_eq!(rows[0].1.total_size, 0);
        assert_eq!(rows[0].1.latest_create_time, 0.0);
    }

    #[test]
    fn test_aggregate_empty_input_is_empty() {
        assert!(aggregate(Vec::new(), &HashMap::new(), &HashMap::new()).is_empty());
    }
}
