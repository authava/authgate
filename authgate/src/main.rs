use authgate::admin::{
    create_admin_router, create_route, delete_route, get_route, is_admin_api_enabled, list_routes,
    update_route,
};
use authgate::auth::AuthService;
use authgate::config::ConfigManager;
use authgate::matcher::RouteMatcher;
use authgate::proxy::{handle_forward_auth, AppState};
use axum::{routing::get, Router};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::prelude::*;

#[cfg(feature = "postgres")]
async fn run_migrations_if_postgres() -> anyhow::Result<()> {
    let backend = std::env::var("AUTHGATE_CONFIG_BACKEND").unwrap_or_else(|_| "json".into());
    if backend == "postgres" {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set when using Postgres backend");
        let pool = sqlx::PgPool::connect(&database_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        tracing::info!("Postgres migrations applied successfully.");

        bootstrap_seeds_if_needed(&pool).await?;
    }
    Ok(())
}

#[cfg(feature = "postgres")]
async fn bootstrap_seeds_if_needed(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    use serde_json::json;

    // Seed auth_config
    let session_url = std::env::var("AUTHGATE_BOOTSTRAP_SESSION_URL").ok();
    let login_redirect = std::env::var("AUTHGATE_BOOTSTRAP_LOGIN_REDIRECT").ok();
    let cookie_name = std::env::var("AUTHGATE_BOOTSTRAP_COOKIE_NAME").unwrap_or_else(|_| "session".into());

    if let (Some(session_url), Some(login_redirect)) = (session_url, login_redirect) {
        let exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM auth_config LIMIT 1")
            .fetch_optional(pool)
            .await?;
        if exists.is_none() {
            sqlx::query("INSERT INTO auth_config (session_url, login_redirect, cookie_name) VALUES ($1, $2, $3)")
                .bind(&session_url)
                .bind(&login_redirect)
                .bind(&cookie_name)
                .execute(pool)
                .await?;
            tracing::info!("✅ Seeded auth_config.");
        }
    }

    // Seed routes
    let host = std::env::var("AUTHGATE_BOOTSTRAP_ROUTE_HOST").ok();
    let path = std::env::var("AUTHGATE_BOOTSTRAP_ROUTE_PATH").ok();
    let require_roles = std::env::var("AUTHGATE_BOOTSTRAP_ROUTE_ROLES").unwrap_or_else(|_| "admin".into());

    if let (Some(host), Some(path)) = (host, path) {
        let exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM routes WHERE host = $1 AND path = $2")
            .bind(&host)
            .bind(&path)
            .fetch_optional(pool)
            .await?;
        if exists.is_none() {
            let require = json!({ "roles": [require_roles] });
            sqlx::query("INSERT INTO routes (host, path, require) VALUES ($1, $2, $3)")
                .bind(&host)
                .bind(&path)
                .bind(&require)
                .execute(pool)
                .await?;
            tracing::info!("✅ Seeded route: {}{}", host, path);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "authgate=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting AuthGate");

    if std::env::var("DATABASE_URL").is_err() {
        dotenvy::from_filename(".env").ok();
    }

    #[cfg(feature = "postgres")]
    run_migrations_if_postgres().await?;

    // Initialize configuration manager
    let config_manager = Arc::new(ConfigManager::new());
    config_manager.load_config().await?;

    // Initialize route matcher
    let route_matcher = Arc::new(RouteMatcher::new(config_manager.get_config_ref()));

    // Initialize auth service
    let auth_service = Arc::new(AuthService::new());

    // Create application state
    let app_state = AppState {
        config_manager: config_manager.clone(),
        route_matcher: route_matcher.clone(),
        auth_service: auth_service.clone(),
    };

    // Create the admin router
    let mut admin_router = create_admin_router::<AppState>();

    // Add routes API endpoints if the Admin API is enabled
    #[cfg(feature = "postgres")]
    if is_admin_api_enabled() {
        // Create a separate router for routes API
        let routes_router = Router::new()
            .route("/", get(list_routes).post(create_route))
            .route(
                "/:id",
                get(get_route).put(update_route).delete(delete_route),
            )
            .with_state(Arc::clone(&config_manager));

        // Nest the routes router under /routes
        admin_router = admin_router.nest("/routes", routes_router);
    }

    // Build the application
    let app = Router::new()
        .route("/auth", get(handle_forward_auth))
        .nest("/admin", admin_router)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Get the port from environment or use default
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(4181);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service(),
    )
    .await?;

    Ok(())
}
