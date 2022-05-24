use crate::handlers::{
    accept_invitation, available_users, create_avatar, create_project, deny_invitation,
    get_credentials, get_invitations, get_project, get_user_projects, invite_user, login, me,
    register,
};
use actix_web::web::{self, ServiceConfig};

pub fn project_router(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/projects")
            .route("", web::post().to(create_project))
            .route("", web::get().to(get_user_projects))
            .route(
                "/{project_id}/available_users",
                web::get().to(available_users),
            )
            .route("/{project_id}", web::get().to(get_project))
            .route("/{project_id}/credentials", web::get().to(get_credentials)),
    );
}

pub fn invitation_router(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/invitation")
            .route("/{project_id}/accept", web::post().to(accept_invitation))
            .route("/{project_id}/deny", web::post().to(deny_invitation))
            .route("", web::post().to(invite_user))
            .route("", web::get().to(get_invitations)),
    );
}

pub fn user_router(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/user").route("/me", web::get().to(me)));
}

pub fn avatar_router(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/avatars").route("", web::post().to(create_avatar)));
}

pub fn public_router(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login)),
    );
}
