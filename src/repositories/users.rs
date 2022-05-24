use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Regex},
    options::FindOptions,
    Collection, Database,
};
use std::str::FromStr;

use crate::{errors::AppError, models::*};

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

#[derive(Clone)]
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

    pub async fn create(&self, user: User) -> Result<Bson, AppError> {
        self.collection
            .insert_one(user, None)
            .await
            .map_err(|error| AppError::db_error(error))
            .map(|update_result| update_result.inserted_id)
    }

    pub async fn get(&self, user_id: ObjectId) -> Result<User, AppError> {
        self.collection
            .find_one(doc! {"_id": user_id}, None)
            .await
            .map_err(|error| AppError::db_error(error))?
            .ok_or(AppError::not_found_error(user_id.to_string()))
    }

    pub async fn get_by_username(&self, username: &String) -> Result<User, AppError> {
        self.collection
            .find_one(doc! {"username": username}, None)
            .await
            .map_err(|error| AppError::db_error(error))?
            .ok_or(AppError::not_found_error(username))
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

    pub async fn add_project(&self, user_id: ObjectId, project_id: &str) -> Result<(), AppError> {
        let result = self
            .collection
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
            .map(|update_result| update_result.modified_count)?;
        match result {
            1 => Ok(()),
            _ => Err(AppError::db_error("Internal error.")),
        }
    }

    pub async fn get_available_users(
        self,
        project_id: &str,
        username_query: &str,
    ) -> Result<Vec<AvailableUser>, AppError> {
        let username_regex = Regex {
            pattern: format!("^{}", username_query),
            options: String::new(),
        };
        let filter = doc! {"projects": {"$ne": project_id}, "username": username_regex};
        let projection = doc! {"username": 1};
        let options = FindOptions::builder()
            .limit(5)
            .projection(projection)
            .build();
        let mongo_result = self
            .collection
            .clone_with_type::<AvailableUser>()
            .find(filter, options)
            .await
            .map_err(|error| AppError::db_error(error))?;
        mongo_result
            .try_collect()
            .await
            .map(|users| users)
            .map_err(|error| AppError::db_error(error))
    }

    pub async fn add_invitation(self, user_id: ObjectId, project_id: &str) -> Result<(), AppError> {
        let result = self
            .collection
            .update_one(
                doc! {
                    "_id": user_id
                },
                doc! {
                    "$push": { "invitations": project_id }
                },
                None,
            )
            .await
            .map_err(|error| AppError::db_error(error))
            .map(|update_result| update_result.modified_count)?;
        match result {
            1 => Ok(()),
            _ => Err(AppError::db_error("Internal error.")),
        }
    }

    pub async fn remove_invitation(
        self,
        user_id: ObjectId,
        project_id: &str,
    ) -> Result<(), AppError> {
        let result = self
            .collection
            .update_one(
                doc! {
                    "_id": user_id
                },
                doc! {
                    "$pull": { "invitations": project_id }
                },
                None,
            )
            .await
            .map_err(|error| AppError::db_error(error))
            .map(|update_result| update_result.modified_count)?;
        match result {
            1 => Ok(()),
            _ => Err(AppError::db_error("Internal error.")),
        }
    }
}
