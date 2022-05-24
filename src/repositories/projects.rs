use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    Collection, Database,
};
use serde::{Deserialize, Serialize};

use crate::{errors::AppError, models::*};

#[derive(Debug, Serialize, Deserialize)]
pub struct InvitationProjection {
    title: String,
    author: String,
    _id: bson::oid::ObjectId,
    region: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectProjection {
    pub _id: bson::oid::ObjectId,
    pub author: bson::oid::ObjectId,
    pub title: String,
    pub region: String,
    pub api_key: String,
    pub members: Vec<MemberProjection>,
    pub invitations: Vec<String>,
    pub avatars: Vec<Avatar>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberProjection {
    pub username: String,
    pub _id: ObjectId,
}

pub struct ProjectRepository {
    pub database: Database,
    pub collection: Collection<Project>,
}

impl ProjectRepository {
    pub fn new(database: Database) -> Self {
        let collection = database.collection::<Project>("projects");
        Self {
            database,
            collection,
        }
    }

    pub async fn create(&self, project: Project) -> Result<ObjectId, AppError> {
        self.collection
            .insert_one(project, None)
            .await
            .map_err(|error| AppError::db_error(error))
            .map(|insert_result| {
                insert_result
                    .inserted_id
                    .as_object_id()
                    .ok_or(AppError::db_error("Error while parsing ObjectId."))
            })?
    }

    pub async fn get(&self, project_id: ObjectId) -> Result<ProjectProjection, AppError> {
        let pipeline = vec![
            doc! {
                "$match": {
                    "_id": project_id
                }
            },
            doc! {
                "$lookup": {
                    "from": "users",
                    "localField": "members",
                    "foreignField": "_id",
                    "as": "members"
                }
            },
            doc! {
                "$project": {
                    "_id": 1,
                    "author": 1,
                    "title": 1,
                    "api_key": 1,
                    "region": 1,
                    "members": {
                        "_id": 1,
                        "username": 1
                    },
                    "invitations": 1,
                    "avatars": 1
                }
            },
            doc! {
                "$limit": 1
            },
        ];
        let mut mongo_result = self
            .collection
            .aggregate(pipeline, None)
            .await
            .map_err(|error| AppError::db_error(error))?;

        let project = mongo_result
            .next()
            .await
            .ok_or(AppError::not_found_error(project_id.to_string()))?
            .map_err(|error| AppError::db_error(error))?;

        bson::from_document(project)
            .map(|projection| projection)
            .map_err(|error| AppError::db_error(error))
    }

    pub async fn add_user(&self, project_id: ObjectId, user_id: ObjectId) -> Result<(), AppError> {
        let result = self
            .collection
            .update_one(
                doc! {
                    "_id": project_id
                },
                doc! {
                    "$push": { "members": user_id }
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

    pub async fn add_avatar(&self, project_id: ObjectId, avatar: Avatar) -> Result<(), AppError> {
        let result = self
            .collection
            .update_one(
                doc! {
                    "_id": project_id
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
            .await
            .map_err(|error| AppError::db_error(error))
            .map(|update_result| update_result.modified_count)?;
        match result {
            1 => Ok(()),
            _ => Err(AppError::db_error("Internal error.")),
        }
    }

    pub async fn get_user_projects(&self, user_id: ObjectId) -> Result<Vec<Project>, AppError> {
        let result = self
            .collection
            .find(doc! {"members": user_id}, None)
            .await
            .map_err(|error| AppError::db_error(error))?;
        result
            .try_collect()
            .await
            .map_err(|error| AppError::db_error(error))
            .map(|projects| projects)
    }

    pub async fn get_user_invitations(
        &self,
        user_id: ObjectId,
    ) -> Result<Vec<InvitationProjection>, AppError> {
        let pipeline = vec![
            doc! {
                "$match": {
                    "invitations": user_id.to_string()
                }
            },
            doc! {
                "$lookup": {
                    "from": "users",
                    "localField": "author",
                    "foreignField": "_id",
                    "as": "author_informations"
                }
            },
            doc! {
                "$project": {
                    "_id": 1,
                    "title": 1,
                    "region": 1,
                    "author": {"$first": "$author_informations.username" }
                }
            },
        ];
        let mut mongo_result = self
            .collection
            .aggregate(pipeline, None)
            .await
            .map_err(|error| AppError::db_error(error))?;
        let mut results: Vec<InvitationProjection> = Vec::new();
        while let Some(result) = mongo_result.next().await {
            let tmp: InvitationProjection =
                bson::from_document(result.map_err(|error| AppError::db_error(error))?)
                    .map_err(|error| AppError::db_error(error))?;
            results.push(tmp);
        }
        Ok(results)
    }

    pub async fn add_invitation(self, project_id: ObjectId, user_id: &str) -> Result<(), AppError> {
        let result = self
            .collection
            .update_one(
                doc! {
                    "_id": project_id
                },
                doc! {
                    "$push": { "invitations": user_id }
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
        project_id: ObjectId,
        user_id: &str,
    ) -> Result<(), AppError> {
        let result = self
            .collection
            .update_one(
                doc! {
                    "_id": project_id
                },
                doc! {
                    "$pull": { "invitations": user_id }
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

    pub async fn get_secret(self, project_id: ObjectId) -> Result<String, AppError> {
        let filter = doc! {"_id": project_id};
        self.collection
            .find_one(filter, None)
            .await
            .map(|project| project)
            .map_err(|error| AppError::db_error(error))?
            .ok_or(AppError::db_error("xd"))
            .map(|project| project.api_secret)
    }
}
