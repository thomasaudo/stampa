use bcrypt::verify;
use mongodb::bson::{doc, oid::ObjectId};

use crate::models::User;

pub async fn create_user(
    database: &mongodb::Database,
    user: User,
) -> Result<ObjectId, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let insert_result = user_collection.insert_one(user, None).await?;
    let user_id = insert_result.inserted_id.as_object_id();
    match user_id {
        Some(x) => Ok(x),
        None => Err("Can not find user".into()),
    }
}

pub async fn login(
    database: &mongodb::Database,
    username: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let user_collection = database.collection::<User>("users");
    let filter = doc! {"username": username, "password": password};
    let user = user_collection.find_one(filter, None).await?;
    match user {
        Some(x) => Ok(x.username),
        None => Err("Can not login the user.".into()),
    }
}

pub async fn verify_password(
    password: &str,
    user_password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let verify = verify(&password, user_password)?;
    match verify {
        true => Ok(()),
        false => Err("Can not login the user.".into()),
    }
}
