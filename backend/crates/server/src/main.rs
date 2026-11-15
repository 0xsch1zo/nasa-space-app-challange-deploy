use adapter_http::{router, AppState};
use server::{build, init_tracing, prepare_database, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let config = Config::from_env()?;
    let pool = adapter_postgres::connect(&config.database_url).await?;
    let container = build(pool.clone(), &config);
    prepare_database(&pool, &container.catalog).await?;

    let state = AppState {
        auth: container.auth,
        crew: container.crew,
        health: container.health,
        alerts: container.alerts,
        wellbeing: container.wellbeing,
    };
    let app = router(state, &config.cors_origins);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "astro-health API listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
