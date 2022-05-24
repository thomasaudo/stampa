use actix_web::{web, HttpResponse, Responder};

use crate::{errors::AppError, repositories::UserRepository, utils::Claims, AppState};

pub async fn me(
    app: web::Data<AppState>,
    claims: Option<web::ReqData<Claims>>,
) -> Result<impl Responder, AppError> {
    let user_id = claims.unwrap().id;
    let user = UserRepository::new(app.database.clone()).get(user_id).await;
    user.map(|user| HttpResponse::Ok().json(user))
}
