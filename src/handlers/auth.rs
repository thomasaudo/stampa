use actix_web::{web, HttpResponse, Responder, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    avatars::AvatarService,
    cloud::CloudClient,
    errors::AppError,
    models::User,
    repositories::UserRepository,
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
) -> Result<impl Responder, AppError> {
    println!("Registering new user");
    let username = &user.username;
    let password = &user.password;
    let user_id = ObjectId::new();
    let hashed_password =
        hash(password.as_str(), DEFAULT_COST).map_err(|error| AppError::db_error(error))?;
    let user_avatar = AvatarService::generate_avatar(&user_id.to_string(), &username[0..2])?;
    println!("Avatar generated");
    let avatar_url = CloudClient::new_application_client()?
        .put_object(&user_avatar, &user_id.to_string())
        .await?;
    println!("Avatar stored");
    UserRepository::new(app.database.clone())
        .create(User {
            id: user_id,
            username: username.to_string(),
            password: hashed_password,
            projects: Vec::new(),
            invitations: Vec::new(),
            avatar: avatar_url,
        })
        .await
        .map(|_| {
            encode_jwt(Claims {
                exp: (Utc::now() + Duration::days(365)).timestamp() as usize,
                sub: user_id.to_string(),
                id: user_id,
            })
            .map_err(|error| AppError::db_error(error))
            .map(|jwt_token| HttpResponse::Ok().json(RegisterResponse { token: jwt_token }))
        })
}

pub async fn login(
    app: web::Data<AppState>,
    user: web::Json<LoginPayload>,
) -> Result<impl Responder, AppError> {
    let username = &user.username;
    let password = &user.password;

    let user_doc = UserRepository::new(app.database.clone())
        .get_by_username(username)
        .await?;

    let expiration = Utc::now() + Duration::days(365);

    let result =
        verify(&password, &user_doc.password).map_err(|error| AppError::db_error(error))?;

    match result {
        true => encode_jwt(Claims {
            exp: expiration.timestamp() as usize,
            sub: user_doc.id.to_string(),
            id: user_doc.id,
        })
        .map_err(|error| AppError::db_error(error))
        .map(|token| HttpResponse::Ok().json(RegisterResponse { token: token })),
        false => Err(AppError::login_error(username)),
    }
}
