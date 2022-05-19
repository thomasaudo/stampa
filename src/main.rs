use actix_web::middleware::Logger;
use actix_web::{web, App, HttpMessage, HttpServer};
use actix_web_httpauth::extractors::AuthenticationError;
use stampa::AppState;
use std::net::TcpListener;

use actix_web::dev::ServiceRequest;
use actix_web::Error;
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::middleware::HttpAuthentication;

use env_logger::Env;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use stampa::handlers::{
    accept_invitation, available_users, create_avatar, create_project, deny_invitation,
    get_credentials, get_invitations, get_project, get_user_projects, invite_user, login, me,
    register,
};
use stampa::utils::Claims;

async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app_config = stampa::config::Config::from_env().unwrap();

    let database = app_config.connect_mongo().await.unwrap();

    let app_state = web::Data::new(AppState { database });

    let address = format!("{}:{}", app_config.host, app_config.port);
    let listener = TcpListener::bind(address.to_string())?;

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let _server = HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(validator);
        App::new()
            .wrap(Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .app_data(app_state.clone())
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .wrap(actix_cors::Cors::permissive())
                    .route("/me", web::get().to(me))
                    .service(
                        web::scope("/projects")
                            .route("", web::post().to(create_project))
                            .route("", web::get().to(get_user_projects))
                            .route(
                                "/{project_id}/available_users",
                                web::get().to(available_users),
                            )
                            .route("/{project_id}", web::get().to(get_project)),
                    )
                    .service(web::scope("/avatars").route("", web::post().to(create_avatar)))
                    .service(
                        web::scope("/invitation")
                            .route("/{project_id}/accept", web::post().to(accept_invitation))
                            .route("/{project_id}/deny", web::post().to(deny_invitation))
                            .route("", web::post().to(invite_user))
                            .route("", web::get().to(get_invitations)),
                    )
                    .service(
                        web::scope("/credentials")
                            .route("/{project_id}", web::get().to(get_credentials)),
                    ),
            )
    })
    .listen(listener)?
    .run()
    .await;

    Ok(())
}
