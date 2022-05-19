pub async fn get_crendentials(
    app: web::Data<AppState>,
    user: web::Json<LoginPayload>,
) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let username = &user.username;
    let password = &user.password;

    let user_doc = get_user_by_username(&app.database, username.to_string()).await?;

    let expiration = Utc::now() + Duration::days(365);
    
    match verify_password(&password, &user_doc.password).await {
        Ok(_) => {
            let jwt_token = encode_jwt(Claims {
                exp: expiration.timestamp() as usize,
                sub: user_doc.id.to_string(),
            })?;
            Ok(web::Json(RegisterResponse { token: jwt_token }))
        }
        Err(error) => Err(error.into()),
    }
}