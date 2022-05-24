use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use std::net::TcpListener;

use crate::middlewares::validator;
use crate::routers::{
    avatar_router, invitation_router, project_router, public_router, user_router,
};
use crate::AppState;

pub fn run(listener: TcpListener, app_state: Data<AppState>) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
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
    .run();
    Ok(server)
}
