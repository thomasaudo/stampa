use std::{io::Read, str::FromStr};

use rusoto_core::{ByteStream, Region};
use rusoto_s3::{
    CreateBucketConfiguration, CreateBucketRequest, GetObjectRequest, PutObjectRequest, S3Client,
    S3,
};

use crate::errors::AppError;

pub struct CloudClient {
    s3: S3Client,
    bucket_name: String,
    region: String,
}

impl CloudClient {
    pub fn new(bucket_name: String, region: String) -> Result<CloudClient, AppError> {
        let s3_region = Region::from_str(region.as_str())
            .map_err(|_| AppError::s3_error("Can not load region."))?;
        Ok(CloudClient {
            region,
            bucket_name,
            s3: S3Client::new(s3_region),
        })
    }

    pub async fn create_bucket(bucket_name: String, region: String) -> Result<String, AppError> {
        let s3_region =
            Region::from_str(region.as_str()).map_err(|error| AppError::s3_error(error))?;
        let s3 = S3Client::new(s3_region);
        let location = match region.as_str() {
            "us-east-1" => None,
            _ => Some(CreateBucketConfiguration {
                location_constraint: Some(region),
            }),
        };
        s3.create_bucket(CreateBucketRequest {
            bucket: bucket_name.clone(),
            create_bucket_configuration: location,
            ..Default::default()
        })
        .await
        .map_err(|error| AppError::s3_error(error))
        .map(|_| bucket_name)
    }

    pub fn url(&self, key: &str) -> String {
        format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.bucket_name, self.region, key
        )
    }

    pub async fn put_object(&self, path: &str, key: &str) -> Result<String, AppError> {
        let mut file = std::fs::File::open(path)
            .map_err(|_| AppError::fs_error("Can not read temporary avatar."))?;
        let mut contents: Vec<u8> = Vec::new();
        let _ = file.read_to_end(&mut contents);

        let put_request = PutObjectRequest {
            acl: Some("public-read".to_string()),
            bucket: self.bucket_name.to_owned(),
            key: key.to_owned(),
            body: Some(contents.into()),
            ..Default::default()
        };

        self.s3
            .put_object(put_request)
            .await
            .map_err(|error| {
                println!("{}", error);
                AppError::s3_error(error)
            })
            .map(|_| self.url(key))
    }

    pub async fn get_object(&self, key: &str) -> Result<ByteStream, AppError> {
        let get_request = GetObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: key.to_owned(),
            ..Default::default()
        };

        self.s3
            .get_object(get_request)
            .await
            .map_err(|_| AppError::s3_error("Can not get object."))
            .map(|response| {
                response
                    .body
                    .ok_or(AppError::s3_error("Can not get the object body."))
            })?
    }
}
