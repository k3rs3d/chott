use std::sync::{Arc, Mutex};

use actix_web::{HttpResponse, Responder, web};
use tera::{Context, Tera};
use tracing::{error, info, instrument};

use crate::actor::{Actor, ActorManager};
use crate::environment::EnvironmentManager;
use crate::error::AppError;
use crate::pages::{PageGraph, valid_move};
use crate::session::{
    SESSION_KEY, UserAction, UserSession, get_or_create_user_session, set_user_session,
};
// TODO: refactor
#[instrument(skip(tera, pages, session, actor_manager, environment_manager, form))] // tracing 
pub async fn index_handler(
    tera: web::Data<Tera>,
    pages: web::Data<Arc<PageGraph>>,
    session: actix_session::Session,
    actor_manager: web::Data<Arc<Mutex<ActorManager>>>,
    environment_manager: web::Data<EnvironmentManager>,
    form: Option<web::Form<UserAction>>,
) -> impl Responder {
    info!(
        "Serving page handler for session {:?}",
        session.get::<UserSession>(SESSION_KEY)
    );

    // Retrieve or create a user session (hardcoded start at palette-town)
    let mut user_session = get_or_create_user_session(&session, "small-town")?;

    // Handle navigation action
    if let Some(action) = form {
        if let Some(conn) = valid_move(&user_session.current_page, &action.go_to, &pages).await {
            info!("User session {} is moving {}", SESSION_KEY, action.go_to);
            user_session.current_page = conn.target.clone();
            set_user_session(&session, &user_session);
        } else {
            error!("Tried invalid direction {}", action.go_to);
            return Err(AppError::SessionError("Invalid direction!".to_string()));
        }
    }

    // Find the current page
    let page = pages
        .get(&user_session.current_page)
        .ok_or_else(|| AppError::PageNotFound(user_session.current_page.to_string()))?;

    // Get environment data for this page
    let environment = environment_manager
        .get_environment_for_page(&page.id)
        .await?;

    let actor_manager_ref = actor_manager
        .get_ref()
        .lock()
        .expect("Failed to lock Mutex");
    let actors_here: Vec<&Actor> = actor_manager_ref
        .actors
        .values()
        .filter(|a| a.location == page.id && a.state.awake) // Show only awake actors, optionally filter more
        .collect();

    // Build template context
    let mut ctx = Context::new();
    ctx.insert("page", page);
    ctx.insert("environment", &environment);
    ctx.insert("npcs", &actors_here);

    let html = tera.render(&page.template, &ctx)?;
    Ok(HttpResponse::Ok().body(html))
}
