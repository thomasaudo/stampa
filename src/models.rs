use mongodb::bson::{self, oid::ObjectId};
use serde::{Deserialize, Serialize};

pub trait Print {
    fn print_informations(&self);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Avatar {
    pub _id: bson::oid::ObjectId,
    pub name: String,
    pub mime_type: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub username: String,
    pub password: String,
    pub avatar: String,
    pub projects: Vec<String>,
    pub invitations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableUser {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Invitation {
    #[serde(rename = "_id")]
    pub _id: String,
    pub author: String,
    pub region: String,
    pub title: String,
}

impl Print for User {
    fn print_informations(&self) {
        println!(
            "[{}] projects: {}, invitations: {}",
            self.username,
            self.projects.len(),
            self.invitations.len()
        );
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub author: ObjectId,
    pub title: String,
    pub api_key: String,
    pub api_secret: String,
    pub region: String,
    pub members: Vec<ObjectId>,
    pub invitations: Vec<String>,
    pub avatars: Vec<Avatar>,
}

impl Print for Project {
    fn print_informations(&self) {
        println!("[{}] author: {}", self.title, self.author);
    }
}
