use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use env_logger::Env;
use std::net::TcpListener;

use stampa::middlewares::validator;
use stampa::routers::{
    avatar_router, invitation_router, project_router, public_router, user_router,
};
use stampa::AppState;

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
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .wrap(actix_cors::Cors::permissive())
                    .configure(user_router)
                    .configure(project_router)
                    .configure(invitation_router)
                    .configure(avatar_router),
            )
            .configure(public_router)
    })
    .listen(listener)?
    .run()
    .await;

    Ok(())
}
