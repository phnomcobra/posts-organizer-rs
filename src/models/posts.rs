/// An encrypted post record stored in the local `posts` collection.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Post {
    /// Base64-encoded body..
    pub b64body: String,
    /// Base64-encoded nonce used to encrypt the key.
    pub headers: Vec<(String, String)>,
    /// Base64-encoded authentication tag for the ciphertext.
    pub tag: Option<String>,
    /// Unix timestamp when the key was created.
    #[serde(default)]
    pub create_time: f64,
    /// Unix timestamp when the key expires.
    #[serde(default)]
    pub expire_time: f64,
    /// Key category.
    #[serde(default)]
    pub r#type: KeyType,
    /// UUID of the peer associated with the key, if peer-specific.
    pub agtuuid: Option<String>,
    /// UUID identifying this key in the collection.
    pub objuuid: Option<String>,
    /// UUID identifying the collection containing this record.
    pub coluuid: Option<String>,
}