use actix_web::{web, Responder};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    cloud::CloudClient,
    database::{add_avatar, get_project},
    models::Avatar,
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
) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let project = get_project(&app.database, avatar.project.clone())
        .await
        .unwrap();
    let _id = ObjectId::new();
    let key = _id.to_string();
    // TODO: Replace split() by find().
    let base64split = avatar.image.split(",").collect::<Vec<&str>>();
    let image_extension = base64split[0].split(";").collect::<Vec<&str>>()[0]
        .split("/")
        .collect::<Vec<&str>>()[1];
    let filepath = format!("./tmp/{}.{}", key, image_extension);
    let image_body = base64split[1];

    let decoded_avatar = base64::decode(image_body).unwrap();
    let tmp_image = image::load_from_memory(&decoded_avatar).unwrap();
    tmp_image.save(&filepath).unwrap();

    let url = CloudClient::new(project.id.to_string(), project.region)
        .put_object(&filepath, format!("{}.{}", &key, image_extension).as_str())
        .await;

    std::fs::remove_file(&filepath).unwrap();

    let new_avatar = Avatar {
        _id,
        mime_type: image_extension.to_string(),
        name: avatar.name.to_string(),
        url: url.to_string(),
    };

    let b = add_avatar(&app.database, &avatar.project, new_avatar.clone())
        .await
        .unwrap();

    match b {
        true => Ok(web::Json(new_avatar)),
        false => Err("Can not add the avatar to the specified project.".into()),
    }
}
