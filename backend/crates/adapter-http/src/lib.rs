//! HTTP driving adapter (axum). Translates requests into use-case calls and
//! results into JSON. Contains no business or authorization rules: those live
//! in the application layer, which receives the authenticated `Principal`.

mod dto;
mod error;
mod extract;
mod handlers;

use std::sync::Arc;

use application::ports::inbound::{
    AlertUseCase, AuthUseCase, CrewUseCase, HealthUseCase, WellbeingUseCase,
};
use axum::http::{header, HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthUseCase>,
    pub crew: Arc<dyn CrewUseCase>,
    pub health: Arc<dyn HealthUseCase>,
    pub alerts: Arc<dyn AlertUseCase>,
    pub wellbeing: Arc<dyn WellbeingUseCase>,
}

pub fn router(state: AppState, cors_origins: &[String]) -> Router {
    let api = Router::new()
        .route("/health", get(handlers::liveness))
        .route("/auth/login", post(handlers::login))
        .route("/me", get(handlers::me))
        .route("/metrics", get(handlers::metric_catalog))
        .route("/metric-groups", get(handlers::metric_groups))
        .route("/astronauts", get(handlers::list_crew))
        .route("/astronauts/{id}", get(handlers::get_astronaut))
        .route("/astronauts/{id}/overview", get(handlers::overview))
        .route("/astronauts/{id}/alerts", get(handlers::alerts))
        .route(
            "/astronauts/{id}/wellbeing-note",
            get(handlers::wellbeing_note),
        )
        .route(
            "/astronauts/{id}/metrics/{code}/series",
            get(handlers::series),
        );

    let origins: Vec<HeaderValue> = cors_origins.iter().filter_map(|o| o.parse().ok()).collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    Router::new()
        .nest("/api", api)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
