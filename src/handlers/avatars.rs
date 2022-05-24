use std::str::FromStr;

use actix_web::{web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    cloud::CloudClient,
    database::{ProjectRepository, UserRepository},
    errors::AppError,
    models::Avatar,
    utils::Claims,
    AppState,
};

#[derive(Serialize, Deserialize)]
pub struct AvatarUpload {
    name: String,
    project: String,
    image: String,
}

pub async fn create_avatar(
    avatar: web::Json<AvatarUpload>,
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
) -> Result<impl Responder, AppError> {
    let user_id = claims.unwrap().id;
    let project_object_id =
        ObjectId::from_str(&avatar.project).map_err(|error| AppError::db_error(error))?;
    let repository = ProjectRepository::new(app.database.clone());
    let avatar_id = ObjectId::new();
    let key = avatar_id.to_string();

    UserRepository::new(app.database.clone())
        .in_project(user_id, &avatar.project)
        .await?;

    let project = repository
        .get(project_object_id)
        .await
        .map_err(|error| AppError::db_error(error))?;

    let base64split = avatar.image.split(",").collect::<Vec<&str>>();
    let image_extension = base64split[0].split(";").collect::<Vec<&str>>()[0]
        .split("/")
        .collect::<Vec<&str>>()[1];
    let filepath = format!("./tmp/{}.{}", key, image_extension);
    let image_body = base64split[1];

    let decoded_avatar = base64::decode(image_body).map_err(|error| AppError::fs_error(error))?;
    let tmp_image =
        image::load_from_memory(&decoded_avatar).map_err(|error| AppError::fs_error(error))?;
    tmp_image
        .save(&filepath)
        .map_err(|error| AppError::fs_error(error))?;

    let url = CloudClient::new(avatar.project.clone(), project.region)?
        .put_object(&filepath, format!("{}.{}", &key, image_extension).as_str())
        .await?;

    std::fs::remove_file(&filepath)
        .map_err(|_| AppError::fs_error("Can not delete temporary avatar."))?;

    let new_avatar = Avatar {
        _id: avatar_id,
        mime_type: image_extension.to_string(),
        name: avatar.name.to_string(),
        url: url.to_string(),
    };

    repository
        .add_avatar(project_object_id, new_avatar)
        .await
        .map(|_| HttpResponse::Ok().json(avatar))
}
