use actix_web::Error;
use actix_web::{dev::ServiceRequest, HttpMessage};
use actix_web_httpauth::extractors::{
    bearer::{BearerAuth, Config},
    AuthenticationError,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::utils::Claims;

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let token = credentials.token();
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(_token) => {
            req.extensions_mut().insert(_token.claims);
            Ok(req)
        }
        Err(_e) => Err(AuthenticationError::from(config).into()),
    }
}
