use warp::Filter;
use dotenv::dotenv;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn encode_jwt(user_id: String) -> String {
    dotenv().ok(); // Load .env file
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = 10000000000; // Example expiration
    let claims = Claims { sub: user_id, exp: expiration };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap()
}

pub fn decode_jwt(token: &str) -> bool {
    dotenv().ok(); // Load .env file
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    decode::<Claims>(&token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default()).is_ok()
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    dotenv().ok(); // Ensure .env variables are loaded
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
}

#[derive(Debug)]
pub struct InvalidJwt;

impl warp::reject::Reject for InvalidJwt {}

pub fn with_auth() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization")
        .and_then(|token: String| async move {
            verify_jwt(&token)
                .map(|_| token) // Return the token itself if valid
                .map_err(|_| warp::reject::custom(crate::InvalidJwt))
        })
}