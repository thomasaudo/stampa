use std::str::FromStr;

use actix_web::{web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

use crate::{
    cloud::CloudClient,
    errors::AppError,
    models::Project,
    repositories::{ProjectRepository, UserRepository},
    utils::{generate_credentials, Claims},
    AppState,
};

#[derive(Deserialize)]
pub struct ProjectPayload {
    title: String,
    region: String,
}

pub async fn get_project(
    app: web::Data<AppState>,
    path: web::Path<String>,
    claims: Option<web::ReqData<Claims>>,
) -> Result<impl Responder, AppError> {
    let user_id = claims.unwrap().id;
    let project_id = path.to_string();
    let project_object_id =
        ObjectId::from_str(&project_id).map_err(|error| AppError::db_error(error))?;

    UserRepository::new(app.database.clone())
        .in_project(user_id, &project_id)
        .await?;

    let project = ProjectRepository::new(app.database.clone())
        .get(project_object_id)
        .await;
    project.map(|result| HttpResponse::Ok().json(result))
}

pub async fn get_user_projects(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
) -> Result<impl Responder, AppError> {
    let user_id = claims.unwrap().id;
    ProjectRepository::new(app.database.clone())
        .get_user_projects(user_id)
        .await
        .map(|projects| HttpResponse::Ok().json(projects))
}

pub async fn get_invitations(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
) -> Result<impl Responder, AppError> {
    let user_id = claims.unwrap().id;
    ProjectRepository::new(app.database.clone())
        .get_user_invitations(user_id)
        .await
        .map(|invitations| HttpResponse::Ok().json(invitations))
}

pub async fn create_project(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
    project: web::Json<ProjectPayload>,
) -> Result<impl Responder, AppError> {
    let user_id = claims.expect("No user_id").id;
    let project_id = ObjectId::new();
    let region = project.region.clone();
    let (api_key, api_secret) = generate_credentials().await;
    let project = Project {
        id: project_id,
        author: user_id,
        api_key,
        api_secret,
        avatars: Vec::new(),
        invitations: Vec::new(),
        members: vec![user_id],
        region: region.clone(),
        title: project.title.to_string(),
    };
    ProjectRepository::new(app.database.clone())
        .create(project.clone())
        .await
        .map_err(|error| AppError::db_error(error))?;
    UserRepository::new(app.database.clone())
        .add_project(user_id, &project_id.to_string())
        .await
        .map_err(|error| AppError::db_error(error))?;
    CloudClient::create_bucket(project_id.to_string(), region)
        .await
        .map_err(|error| error)
        .map(|_| HttpResponse::Ok().json(project))
}

#[derive(Deserialize)]
pub struct AvailableUserQuery {
    username: String,
}

/**
 * TODO: Move this handler in user handlers ?
 */
pub async fn available_users(
    app: web::Data<AppState>,
    path: web::Path<String>,
    claims: Option<web::ReqData<Claims>>,
    query: web::Query<AvailableUserQuery>,
) -> Result<impl Responder, AppError> {
    let user_id = claims.expect("No user_id").id;
    let project_id = path.to_string();
    UserRepository::new(app.database.clone())
        .in_project(user_id, &project_id)
        .await?;
    UserRepository::new(app.database.clone())
        .get_available_users(&project_id, &query.username)
        .await
        .map(|users| HttpResponse::Ok().json(users))
}

#[derive(Deserialize)]
pub struct InviteUserPayload {
    username: String,
    project: String,
}

pub async fn invite_user(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
    invitation: web::Json<InviteUserPayload>,
) -> Result<impl Responder, AppError> {
    let user_id = claims.expect("No user_id").id;
    let project_id = &invitation.project;
    let project_object_id =
        ObjectId::from_str(&project_id).map_err(|error| AppError::db_error(error))?;

    UserRepository::new(app.database.clone())
        .in_project(user_id, &project_id)
        .await?;
    ProjectRepository::new(app.database.clone())
        .add_invitation(project_object_id, invitation.username.to_string().as_str())
        .await
        .map(|_| HttpResponse::Ok())
    // TODO: Add invitation to user
}

pub async fn accept_invitation(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let user_id = &claims.expect("No user_id").id;
    let project_id = path.to_string();
    let project_object_id =
        ObjectId::from_str(&project_id).map_err(|error| AppError::db_error(error))?;

    let user_repository = UserRepository::new(app.database.clone());
    let project_repository = ProjectRepository::new(app.database.clone());

    user_repository.add_project(*user_id, &project_id).await?;
    user_repository
        .remove_invitation(*user_id, &project_id)
        .await?;
    project_repository
        .add_user(project_object_id, *user_id)
        .await?;
    project_repository
        .remove_invitation(project_object_id, user_id.to_string().as_str())
        .await?;

    ProjectRepository::new(app.database.clone())
        .get(project_object_id)
        .await
        .map_err(|error| AppError::db_error(error))
        .map(|project| HttpResponse::Ok().json(project))
}

pub async fn deny_invitation(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let user_id = &claims.expect("No user_id").id;
    let project_id = path.to_string();
    let project_object_id =
        ObjectId::from_str(&project_id).map_err(|error| AppError::db_error(error))?;

    UserRepository::new(app.database.clone())
        .remove_invitation(*user_id, &project_id)
        .await?;
    ProjectRepository::new(app.database.clone())
        .remove_invitation(project_object_id, user_id.to_string().as_str())
        .await
        .map(|_| HttpResponse::Ok())
}

pub async fn get_credentials(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let user_id = &claims.expect("No user_id").id;
    let project_id = path.to_string();
    let project_object_id =
        ObjectId::from_str(&project_id).map_err(|error| AppError::db_error(error))?;
    UserRepository::new(app.database.clone())
        .in_project(*user_id, &project_id)
        .await?;
    ProjectRepository::new(app.database.clone())
        .get_secret(project_object_id)
        .await
        .map(|secret| HttpResponse::Ok().json(secret))
}
