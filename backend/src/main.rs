use std::sync::Arc;

use axum::{
    Extension, Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    routing::{get, post},
};
use config::Config;
use db::DBClient;
use dotenv::dotenv;
use handler::{create_capsule, get_all_capsules, get_capsule_by_public_id};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tracing_subscriber::filter::LevelFilter;

mod config;
mod db;
mod dtos;
mod errors;
mod handler;

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    dotenv().ok();

    let config = Config::init();

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Connection to the databse is successfull!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the Database: {:?}", err);
            std::process::exit(1);
        }
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);

    let db_client = DBClient::new(pool);
    let app_state = AppState {
        env: config.clone(),
        db_client,
    };

    let app = Router::new()
        .route("/create", post(create_capsule))
        .route("/capsules", get(get_all_capsules))
        .route("/capsule/:public_id", get(get_capsule_by_public_id))
        .layer(Extension(Arc::new(app_state)))
        .layer(cors);

    println!(
        "{}",
        format!("🚀 Server is running on http://localhost:{}", &config.port),
    );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &config.port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
