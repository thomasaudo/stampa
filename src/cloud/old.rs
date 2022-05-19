/**
 * 
async fn get_profile_picture(user_id: web::Path<String>, params: web::Query<ProfilePictureParams>) -> HttpResponse {

    let s3_client = (create_s3_client().await).unwrap();
    let rekognition_client = (create_rekognition_client().await).unwrap();

    let object = (load_s3_image(s3_client, BUCKET_NAME, &user_id).await).expect("Can not load object.");
    let bytes = object.into_bytes();
    let image = (image::load_from_memory_with_format(&bytes, ImageFormat::Png)).expect("Can not decode image.");


    let (h, w) = get_image_dimension(image.borrow());

    let mut final_image = image;

    if params.crop {

        let s3_obj = aws_sdk_rekognition::model::S3Object::builder()
        .bucket(BUCKET_NAME)
        .name(&*user_id)
        .build();

        let face = detect_face(rekognition_client, aws_sdk_rekognition::model::Image::builder().s3_object(s3_obj).build()).await.unwrap();

        final_image = crop_face(final_image, h,w,face);
    }

    final_image = thumbnail_image(final_image, &params.size).await;


    let mut w = Cursor::new(Vec::new());
    final_image
        .write_to(&mut w, ImageOutputFormat::Png)
        .expect("Can't write image.");
    HttpResponse::build(StatusCode::OK)
            .content_type("image/png")
            .body(w.into_inner())
}


#[derive(serde::Deserialize)]
pub struct ProfilePictureParams {
    size: String,
    crop: bool,
}

async fn thumbnail_image(image: DynamicImage, size: &String) -> DynamicImage {
    match size.as_str() {
        "XS" => image.thumbnail(50, 50),
        "MD" => image.thumbnail(100, 100),
        "XL" => image.thumbnail(200, 200),
        _ => image,
    }
}

async fn create_mongo_client() -> Result<Database, mongodb::error::Error> {
    let client = mongodb::Client::with_uri_str("").await?;
    let db = client.database("stampa");
    Ok(db)
}

#[derive(Debug, Serialize, Deserialize)]
struct Project {
    author: String,
    title: String,
}

async fn load_s3_image(
    client: aws_sdk_s3::Client,
    bucket_name: &str,
    key: &str,
) -> Result<AggregatedBytes, aws_sdk_s3::Error> {
    let response = client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await?;
    let data = response.body.collect().await.expect("Cant load object");
    Ok(data)
}

async fn detect_face(
    client: aws_sdk_rekognition::Client,
    image: aws_sdk_rekognition::model::Image,
) -> Result<aws_sdk_rekognition::model::BoundingBox, aws_sdk_rekognition::Error> {
    let resp = client
        .detect_faces()
        .image(image)
        .attributes(aws_sdk_rekognition::model::Attribute::All)
        .send()
        .await;

    Ok(resp.unwrap().face_details.unwrap_or_default()[0]
        .bounding_box()
        .unwrap()
        .to_owned())
}

fn crop_face(
    mut picture: DynamicImage,
    h: u32,
    w: u32,
    bounding_box: aws_sdk_rekognition::model::BoundingBox,
) -> DynamicImage {
    let left = bounding_box.left().unwrap() * w as f32;
    let top = bounding_box.top().unwrap() * h as f32;
    let width = bounding_box.width().unwrap() * w as f32;
    let height = bounding_box.height().unwrap() * h as f32;

    picture.crop(left as u32, top as u32, width as u32, height as u32)
}

fn get_image_dimension(image: &DynamicImage) -> (u32, u32) {
    let image_height = image.height();
    let image_width = image.width();
    (image_height, image_width)
}
**/
