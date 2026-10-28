use serde::{Deserialize, Serialize};

/// An encrypted post record stored in the local `posts` collection.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Post {
    /// Base64-encoded body..
    pub b64_body: String,
    /// Size of the post body in bytes.
    pub size: usize,
    /// HTTP headers associated with the post.
    pub headers: Vec<(String, String)>,
    /// URL path the post was sent to.
    pub path: String,
    /// Unix timestamp when the post was created.
    #[serde(default)]
    pub create_time: f64,
    /// UUID identifying this post in the collection.
    pub objuuid: Option<String>,
    /// UUID identifying the collection containing this record.
    pub coluuid: Option<String>,
}