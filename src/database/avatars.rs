use mongodb::bson::{doc, oid::ObjectId};
use std::str::FromStr;

use crate::models::*;

pub async fn add_avatar(
    database: &mongodb::Database,
    project_id: &String,
    avatar: Avatar,
) -> Result<bool, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    ();
    let update_result = project_collection
        .update_one(
            doc! {
                "_id": ObjectId::from_str(project_id.as_str()).unwrap()
            },
            doc! {
                "$push": { "avatars": doc!{
                    "_id": avatar._id,
                    "name": avatar.name,
                     "mime_type": avatar.mime_type,
                     "url": avatar.url
                } }
            },
            None,
        )
        .await?;
    match update_result.modified_count {
        1 => Ok(true),
        _ => Err("Can not create avatar".into()),
    }
}

pub async fn get_avatar(
    database: &mongodb::Database,
    avatar_id: String,
) -> Result<Avatar, Box<dyn std::error::Error>> {
    let avatar_collection = database.collection::<Avatar>("avatars");
    let filter = doc! {"_id": ObjectId::from_str(avatar_id.as_str())?};
    let avatar = avatar_collection.find_one(filter, None).await?;
    match avatar {
        Some(x) => Ok(x),
        None => Err("Can not find the avatar".into()),
    }
}
