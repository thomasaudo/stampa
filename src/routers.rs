use crate::handlers::{
    accept_invitation, create_avatar, create_project, deny_invitation, get_available_users,
    get_invitations, get_project, get_project_credentials, get_projects, invite_user, login, me,
    register,
};
use actix_web::web::{self, ServiceConfig};

pub fn project_router(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/project")
            // Create a project
            .route("", web::post().to(create_project))
            // Get projects
            .route("", web::get().to(get_projects))
            // Get specific project
            .route("/{project_id}", web::get().to(get_project))
            // Get specific project credentials
            .route(
                "/{project_id}/credentials",
                web::get().to(get_project_credentials),
            )
            // Get specific project available_users
            .route(
                "/{project_id}/available_users",
                web::get().to(get_available_users),
            ),
    );
}

pub fn invitation_router(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/invitation")
            // Invite user
            .route("", web::post().to(invite_user))
            // Get user invitations
            .route("", web::get().to(get_invitations))
            // Accept invitation to a specific project
            .route("/{project_id}/accept", web::post().to(accept_invitation))
            // Deny invitation to a specific project
            .route("/{project_id}/deny", web::post().to(deny_invitation)),
    );
}

pub fn user_router(cfg: &mut ServiceConfig) {
    // Get user informations
    cfg.service(web::scope("/user").route("", web::get().to(me)));
}

pub fn avatar_router(cfg: &mut ServiceConfig) {
    // Add an avatar
    cfg.service(web::scope("/avatar").route("", web::post().to(create_avatar)));
}

pub fn public_router(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("")
            // Register user
            .route("/register", web::post().to(register))
            // Login user
            .route("/login", web::post().to(login)),
    );
}
