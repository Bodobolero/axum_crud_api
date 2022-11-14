//! Run with
//!
//! ```not_rust
//! cd examples && cargo run -p example-hello-world
//! ```

use axum::{
    extract::Extension,
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// sqlx
use anyhow::Context;
use sqlx::{Pool, Sqlite, sqlite::{SqlitePoolOptions, SqliteConnectOptions, SqliteJournalMode}, ConnectOptions};
use sqlx::Connection;
use std::str::FromStr;

mod controllers;
mod models;

// openAPI doc
use utoipa::{
    OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
   
    #[derive(OpenApi)]
    #[openapi(
        paths(
            controllers::task::all_tasks,
            controllers::task::new_task,
            controllers::task::task,
            controllers::task::update_task,
            controllers::task::delete_task,
            
        ),
        components(
            schemas(models::task::Task,models::task::NewTask, models::task::UpdateTask)
        ),
        tags(
            (name = "task", description = "Tasks management API")
        )
    )]
    struct ApiDoc;

    init_tracing();
    
    let pool = prepare_database().await?;

    // build our application with a route
    let app = Router::new()
         // openAPI doc under: http://127.0.0.1:3000/swagger-ui
        .merge(SwaggerUi::new("/swagger-ui/*tail").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route("/hello", get(root))
        .route("/tasks", get(controllers::task::all_tasks))
        .route("/tasks", post(controllers::task::new_task))
        .route("/tasks/:id", get(controllers::task::task))
        .route("/tasks/:id", put(controllers::task::update_task))
        .route("/tasks/:id", delete(controllers::task::delete_task))
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World"
}

fn init_tracing(){
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("tower_http=debug,axum_crud_api=debug")
                .unwrap_or_else(|_| "axum_crud_api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn prepare_database() -> anyhow::Result<Pool<Sqlite>> {
    // sqlx-sqlite database pool
    use std::env;
    let database_url = env::var("DATABASE_URL").unwrap_or("sqlite:tasks.db".to_string());

    // create database if it does not exist 
    let conn = SqliteConnectOptions::from_str(&database_url)?
    .journal_mode(SqliteJournalMode::Wal).create_if_missing(true)
    .connect().await?;
    conn.close();

    // prepare connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await
        .context("could not connect to database_url")?;

    // prepare schema in db if it does not yet exist
    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
