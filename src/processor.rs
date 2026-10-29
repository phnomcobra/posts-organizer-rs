use actix_web::{HttpRequest, HttpResponse, web::Bytes};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;

use crate::collections::open_posts;
use crate::models::posts::Post;

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

/// Catch-all POST handler.  Ported from postorganizer's `router/catchall.py`
/// `post` route: stores the request body and path as a new post in the posts
/// collection, then echoes the body back to the caller.
pub async fn catch_all_post_handler(
    req: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    let post = Post {
        b64_body: STANDARD.encode(&body),
        size: body.len(),
        path: req.path().to_string(),
        create_time: Utc::now().timestamp_millis(),
        objuuid: None,
        coluuid: None,
    };

    match open_posts().and_then(|posts| posts.upsert_object(post)) {
        Ok(_) => {
            log::info!("{} bytes to {}", body.len(), req.path());
            HttpResponse::Ok().body(body)
        }
        Err(error) => {
            log::error!("{} raised {}", req.path(), error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Catch-all GET handler.  Ported from postorganizer's `router/catchall.py`
/// `get` route: retrieves all posts from the posts collection and returns them as JSON.
pub async fn catch_all_get_handler() -> HttpResponse {
    match open_posts() {
        Ok(posts) => match posts.find_all() {
            Ok(all_posts) => HttpResponse::Ok().json(all_posts),
            Err(error) => {
                log::error!("Failed to retrieve posts: {}", error);
                HttpResponse::InternalServerError().finish()
            }
        },
        Err(error) => {
            log::error!("Failed to open posts collection: {}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}