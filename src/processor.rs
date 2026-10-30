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
    request: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    let post = Post {
        b64_body: STANDARD.encode(&body),
        size: body.len(),
        path: request.path().to_string(),
        create_time: Utc::now().timestamp_millis(),
        objuuid: None,
        coluuid: None,
    };

    match open_posts().and_then(|posts| posts.upsert_object(post)) {
        Ok(_) => {
            log::info!("{} bytes to {}", body.len(), request.path());
            HttpResponse::Ok().body(body)
        }
        Err(error) => {
            log::error!("{} raised {}", request.path(), error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Catch-all GET handler.  Ported from postorganizer's `router/catchall.py`
/// `get` route: retrieves all posts from the posts collection and returns them as JSON.
pub async fn catch_all_get_handler(request: HttpRequest) -> HttpResponse {
    let collection = match open_posts() {
        Ok(collection) => collection,
        Err(error) => {
            log::error!("Failed to open posts collection: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let posts = match collection.pop_limited(&[("path", &format!("$lt:{}", request.path()))], 1) {
        Ok(posts) => posts,
        Err(error) => {
            log::error!("Failed to pop expired posts: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let Some(post) = posts.first() else {
        return HttpResponse::NotFound().finish();
    };

    let body = match STANDARD.decode(&post.object.b64_body) {
        Ok(body) => body,
        Err(error) => {
            log::error!("Failed to decode post body: {}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok()
        .insert_header(("CREATE_TIME", post.object.create_time.to_string()))
        .body(body)
}