use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Regex},
    options::FindOptions,
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{errors::AppError, models::*};
pub async fn get_projects(
    database: &mongodb::Database,
) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    let mongo_result = project_collection.find(None, None).await?;
    Ok(mongo_result.try_collect().await?)
}

pub async fn get_project_invitations(
    database: &mongodb::Database,
    project_id: String,
) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let filter = doc! {"invitations": project_id.as_str()};
    let mongo_result = user_collection.find(filter, None).await?;
    Ok(mongo_result.try_collect().await?)
}

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

pub async fn get_user_invitation(
    database: &mongodb::Database,
    user_id: String,
) -> Result<Vec<InvitationProjection>, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    let pipeline = vec![
        doc! {
            "$match": {
                "invitations": user_id
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
    let mut mongo_result = project_collection.aggregate(pipeline, None).await.unwrap();
    let mut results: Vec<InvitationProjection> = Vec::new();
    while let Some(result) = mongo_result.next().await {
        let tmp: InvitationProjection = bson::from_document(result?)?;
        results.push(tmp);
    }
    Ok(results)
}

pub async fn get_project_members(
    database: &mongodb::Database,
    project_id: String,
) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let filter = doc! {"projects": project_id.as_str()};
    let mongo_result = user_collection.find(filter, None).await?;
    Ok(mongo_result.try_collect().await?)
}

pub async fn create_project(
    database: &mongodb::Database,
    project: Project,
) -> Result<ObjectId, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    //project.author = ObjectId::from_str(project.author).unwrap();
    let insert_result = project_collection.insert_one(project, None).await.unwrap();
    let project_id = insert_result.inserted_id.as_object_id();
    match project_id {
        Some(p) => Ok(p),
        None => Err("Can not create project".into()),
    }
}

pub async fn add_invitation(
    database: &mongodb::Database,
    user_id: &str,
    project_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    let projectid = ObjectId::from_str(project_id).unwrap();
    let userid = ObjectId::from_str(user_id).unwrap();
    ();
    project_collection
        .update_one(
            doc! {
                "_id": projectid
            },
            doc! {
                "$push": { "invitations": user_id }
            },
            None,
        )
        .await
        .unwrap();
    let user_collection = database.collection::<User>("users");
    user_collection
        .update_one(
            doc! {
                "_id": userid
            },
            doc! {
                "$push": { "invitations": project_id }
            },
            None,
        )
        .await
        .unwrap();
    Ok(())
}

pub async fn delete_invitation_from_project(
    database: &mongodb::Database,
    user_id: &String,
    project_id: &String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    let update_result = project_collection
        .update_one(
            doc! {
                "_id": ObjectId::from_str(project_id.as_str())?
            },
            doc! {
                "$pull": { "invitations": user_id }
            },
            None,
        )
        .await?;
    match update_result.modified_count {
        1 => Ok(true),
        _ => Err("Can not delete invitation".into()),
    }
}

pub async fn add_avatar_to_project(
    database: &mongodb::Database,
    avatar_id: String,
    project_id: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    let update_result = project_collection
        .update_one(
            doc! {
                "_id": ObjectId::from_str(project_id.as_str()).unwrap()
            },
            doc! {
                "$push": { "avatars": avatar_id }
            },
            None,
        )
        .await?;
    match update_result.matched_count {
        1 => Ok(true),
        _ => Err("Can not add the avatar to the project.".into()),
    }
}

pub async fn get_available_users(
    database: &mongodb::Database,
    project_id: String,
    username: String,
) -> Result<Vec<AvailableUser>, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let username_regex = Regex {
        pattern: format!("^{}", username),
        options: String::new(),
    };
    let filter = doc! {"projects": {"$ne": project_id.as_str()}, "username": username_regex};
    let projection = doc! {"username": 1};
    let options = FindOptions::builder()
        .limit(5)
        .projection(projection)
        .build();
    let mongo_result = user_collection
        .clone_with_type::<AvailableUser>()
        .find(filter, options)
        .await?;
    let results = mongo_result.try_collect().await?;
    Ok(results)
}

pub async fn add_user_to_project(
    database: &mongodb::Database,
    user_id: String,
    project_id: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    let update_result = project_collection
        .update_one(
            doc! {
                "_id": ObjectId::from_str(project_id.as_str()).unwrap()
            },
            doc! {
                "$push": { "members": ObjectId::from_str(user_id.as_str()).unwrap() }
            },
            None,
        )
        .await?;
    match update_result.matched_count {
        1 => Ok(true),
        _ => Err("Can not add the user to the project.".into()),
    }
}

pub async fn find_project_credentials(
    database: &mongodb::Database,
    project_id: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let project_collection = database.collection::<Project>("projects");
    let filter = doc! {"_id": ObjectId::from_str(&project_id).unwrap()};
    let mongo_result = project_collection.find_one(filter, None).await?;
    Ok(mongo_result.unwrap().api_secret)
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

    pub async fn add_user(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<bool, AppError> {
        self.collection
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
            .map(|update_result| {
                if update_result.modified_count == 0 {
                    true
                } else {
                    false
                }
            })
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
}
