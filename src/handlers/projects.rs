use std::str::FromStr;

use actix_web::{web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

use crate::{
    cloud::CloudClient,
    database::{
        add_invitation, add_project_to_user, add_user_to_project, delete_invitation_from_project,
        delete_invitation_from_user, find_project_credentials, get_available_users,
        ProjectRepository, UserRepository,
    },
    errors::AppError,
    models::Project,
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

pub async fn available_users(
    app: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<AvailableUserQuery>,
) -> impl Responder {
    let users = get_available_users(&app.database, path.to_string(), query.username.clone()).await;
    match users {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => {
            println!("{}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct InviteUserPayload {
    username: String,
    project: String,
}

pub async fn invite_user(
    app: web::Data<AppState>,
    invitation: web::Json<InviteUserPayload>,
) -> impl Responder {
    let project_id = invitation.project.clone();
    add_invitation(&app.database, &invitation.username, &project_id)
        .await
        .unwrap();
    HttpResponse::Ok().json({})
}

pub async fn accept_invitation(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
    path: web::Path<String>,
) -> impl Responder {
    let user_id = &claims.expect("No user_id").sub;
    let project_id = path.to_string();
    add_project_to_user(&app.database, user_id.to_string(), project_id.clone())
        .await
        .unwrap();
    add_user_to_project(&app.database, user_id.to_string(), project_id.clone())
        .await
        .unwrap();
    delete_invitation_from_project(&app.database, user_id, &project_id)
        .await
        .unwrap();
    delete_invitation_from_user(&app.database, user_id, &project_id)
        .await
        .unwrap();
    let project = crate::database::get_project(&app.database, project_id).await;
    match project {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => {
            println!("{}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn deny_invitation(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
    path: web::Path<String>,
) -> impl Responder {
    let user_id = &claims.expect("No user_id").sub;
    let project_id = path.to_string();
    delete_invitation_from_project(&app.database, user_id, &project_id)
        .await
        .unwrap();
    delete_invitation_from_user(&app.database, user_id, &project_id)
        .await
        .unwrap();
    let project = crate::database::get_project(&app.database, project_id).await;
    match project {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(err) => {
            println!("{}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_credentials(app: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let secret = find_project_credentials(&app.database, path.to_string()).await;
    match secret {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
