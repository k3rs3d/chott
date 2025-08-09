use actix_files::Files;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::App;
use actix_web::{HttpServer, cookie::Key, web};
use chrono::Timelike;
use std::sync::{Arc, Mutex};
use tera::Tera;
use tracing_subscriber::{
    EnvFilter, fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

use crate::actor::ActorManager;
use crate::environment::WorldTime;
use crate::pages::{PageGraph, load_page_graph};

mod actor;
mod environment;
mod error;
mod handler;
mod pages;
mod session;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // default log level set to debug
    let env_filter = match std::env::var("RUST_LOG") {
        Ok(_) => EnvFilter::from_default_env(),
        Err(_) => EnvFilter::new("debug"),
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .init();

    let tera = Tera::new("templates/*.html").unwrap();
    let page_graph: Arc<PageGraph> = Arc::new(load_page_graph());
    let actor_manager = Arc::new(Mutex::new(ActorManager::new()));
    let environment_manager = environment::EnvironmentManager::new();

    let actor_manager_bg = actor_manager.clone();
    let pages_clone = page_graph.clone();

    // Start background actor tick task
    actix_rt::spawn(async move {
        let mut intvl = actix_rt::time::interval(std::time::Duration::from_secs(2));
        loop {
            intvl.tick().await;
            let tick_result = std::panic::catch_unwind(|| {
                let now = chrono::Local::now();
                let world_time = WorldTime {
                    hour: now.hour() as u8,
                    _minute: now.minute() as u8,
                };
                let mut guard = actor_manager_bg.lock().unwrap();
                guard.tick_some(&world_time, &pages_clone);
            });
            if let Err(panic_info) = tick_result {
                eprintln!("WORLD TICK PANIC! Continuing. Info: {panic_info:?}"); // placeholder
            }
        }
    });

    // cookie session storage
    let secret_key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(page_graph.clone()))
            .app_data(web::Data::new(actor_manager.clone()))
            .app_data(web::Data::new(environment_manager.clone()))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .service(
                web::resource("/")
                    .route(web::get().to(handler::index_handler))
                    .route(web::post().to(handler::index_handler)),
            )
            .service(Files::new("/static", "./static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
