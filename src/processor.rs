use crate::collections::open_posts;

/// Expire posts that have passed their expiration time.
pub fn expire_posts(expiration_time: f64) -> Result<(), Box<dyn std::error::Error>> {
    let posts = open_posts()?;

    let expired = posts.pop(&[("create_time", &format!("$lt:{expiration_time}"))])?;

    log::info!(
        "Expired {} post(s) older than {}",
        expired.len(),
        expiration_time
    );

    Ok(())
}