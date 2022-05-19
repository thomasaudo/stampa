use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::InsertOneResult,
    Collection, Database,
};
use std::str::FromStr;

use crate::{errors::AppError, models::*};

pub async fn get_user(
    database: &mongodb::Database,
    user_id: String,
) -> Result<User, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let filter = doc! {"_id": ObjectId::from_str(user_id.as_str()).unwrap()};
    let user = user_collection.find_one(filter, None).await?;

    match user {
        Some(x) => Ok(x),
        None => Err("Can not find the user".into()),
    }
}

pub async fn get_project(
    database: &mongodb::Database,
    project_id: String,
) -> Result<Project, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<Project>("projects");
    let filter = doc! {"_id": ObjectId::from_str(project_id.as_str()).unwrap()};
    let user = user_collection.find_one(filter, None).await?;

    match user {
        Some(x) => Ok(x),
        None => Err("Can not find the user".into()),
    }
}

pub async fn get_user_by_username(
    database: &mongodb::Database,
    username: String,
) -> Result<User, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let filter = doc! {"username": username};
    let user = user_collection.find_one(filter, None).await?;

    match user {
        Some(x) => Ok(x),
        None => Err("Can not find the user".into()),
    }
}

pub async fn get_user_projects(
    database: &mongodb::Database,
    user_id: String,
) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    let filter = doc! {"members": ObjectId::from_str(user_id.as_str()).unwrap()};
    let mongo_result = project_collection.find(filter, None).await?;
    let results = mongo_result.try_collect().await?;
    Ok(results)
}
pub async fn get_user_invitations(
    database: &mongodb::Database,
    user_id: String,
) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    let filter = doc! {"invitations": user_id.as_str()};
    let mongo_result = project_collection.find(filter, None).await.unwrap();
    let results = mongo_result.try_collect().await?;
    Ok(results)
}

pub async fn add_project_to_user(
    database: &mongodb::Database,
    user_id: String,
    project_id: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let update_result = user_collection
        .update_one(
            doc! {
                "_id": ObjectId::from_str(user_id.as_str()).unwrap()
            },
            doc! {
                "$push": { "projects": project_id }
            },
            None,
        )
        .await?;
    match update_result.matched_count {
        1 => Ok(true),
        _ => Err("Can not add project to the user.".into()),
    }
}

pub async fn add_invitation_to_user(
    database: &mongodb::Database,
    user_id: &str,
    project_id: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let update_result = user_collection
        .update_one(
            doc! {
                "_id": ObjectId::from_str(user_id).unwrap()
            },
            doc! {
                "$push": { "invitations": project_id }
            },
            None,
        )
        .await?;
    match update_result.modified_count {
        1 => Ok(true),
        _ => Err("Can not invite user".into()),
    }
}

pub async fn delete_invitation_from_user(
    database: &mongodb::Database,
    user_id: &String,
    project_id: &String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let update_result = user_collection
        .update_one(
            doc! {
                "_id": ObjectId::from_str(user_id.as_str())?
            },
            doc! {
                "$pull": { "invitations": project_id }
            },
            None,
        )
        .await?;
    match update_result.modified_count {
        1 => Ok(true),
        _ => Err("Can not delete invitation".into()),
    }
}

pub struct UserRepository {
    pub database: Database,
    pub collection: Collection<User>,
}

impl UserRepository {
    pub fn new(database: Database) -> Self {
        let collection = database.collection::<User>("users");
        Self {
            database,
            collection,
        }
    }

    pub async fn create(&self, user: User) -> Result<InsertOneResult, AppError> {
        self.collection
            .insert_one(user, None)
            .await
            .map_err(|error| AppError::db_error(error))
    }

    pub async fn get(&self, user_id: ObjectId) -> Result<User, AppError> {
        self.collection
            .find_one(doc! {"_id": user_id}, None)
            .await
            .map_err(|error| AppError::db_error(error))?
            .ok_or(AppError::not_found_error(user_id.to_string()))
    }

    pub async fn in_project(&self, user_id: ObjectId, project_id: &str) -> Result<(), AppError> {
        self.collection
            .find_one(doc! {"_id": user_id, "projects": project_id}, None)
            .await
            .map_err(|error| AppError::db_error(error))?
            .map_or(
                Err(AppError::not_in_project_error(user_id.to_string())),
                |_| Ok(()),
            )
    }

    pub async fn add_project(&self, user_id: ObjectId, project_id: &str) -> Result<bool, AppError> {
        self.collection
            .update_one(
                doc! {
                    "_id": user_id
                },
                doc! {
                    "$push": { "projects": project_id }
                },
                None,
            )
            .await
            .map_err(|error| AppError::db_error(error))
            .map(|update_result| {
                if update_result.modified_count == 0 {
                    true
                } else {
                    false
                }
            })
    }
}
