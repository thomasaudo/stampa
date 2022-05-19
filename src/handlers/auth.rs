use actix_web::{web, Responder, Result};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    database::{create_user, get_user_by_username, verify_password},
    models::User,
    utils::{encode_jwt, Claims},
    AppState,
};

#[derive(Deserialize)]
pub struct RegisterPayload {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    token: String,
}

pub async fn register(
    app: web::Data<AppState>,
    user: web::Json<RegisterPayload>,
) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let username = &user.username;
    let password = &user.password;
    let hashed_password = hash(password.as_str(), DEFAULT_COST)?;

    let user_id = create_user(
        &app.database,
        User {
            id: ObjectId::new(),
            username: username.to_string(),
            password: hashed_password,
            projects: Vec::new(),
            invitations: Vec::new(),
        },
    )
    .await;

    let expiration = Utc::now() + Duration::days(365);

    match user_id {
        Ok(id) => {
            let jwt_token = encode_jwt(Claims {
                exp: expiration.timestamp() as usize,
                sub: id.to_string(),
                id,
            })?;
            Ok(web::Json(RegisterResponse { token: jwt_token }))
        }
        Err(_) => Err("Can not create the user.".into()),
    }
}

pub async fn login(
    app: web::Data<AppState>,
    user: web::Json<LoginPayload>,
) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let username = &user.username;
    let password = &user.password;

    let user_doc = get_user_by_username(&app.database, username.to_string()).await?;

    let expiration = Utc::now() + Duration::days(365);

    match verify_password(&password, &user_doc.password).await {
        Ok(_) => {
            let jwt_token = encode_jwt(Claims {
                exp: expiration.timestamp() as usize,
                sub: user_doc.id.to_string(),
                id: user_doc.id,
            })?;
            Ok(web::Json(RegisterResponse { token: jwt_token }))
        }
        Err(error) => Err(error.into()),
    }
}
