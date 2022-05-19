use jsonwebtoken::{encode, EncodingKey, Header};
use mongodb::bson::{doc, oid::ObjectId};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub id: ObjectId,
}

pub fn encode_jwt(claims: Claims) -> Result<String, Box<dyn std::error::Error>> {
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    )?;
    Ok(token)
}

pub async fn generate_credentials() -> (String, String) {
    let key = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    let secret = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    (key, secret)
}
