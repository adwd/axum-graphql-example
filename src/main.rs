use axum::{routing::get, Router};
use axum_graphql_example::graphql_schema;
use dotenvy::from_filename;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    match env::var("CI").ok() {
        Some(_) => {
            // CIのときはPostgreSQLを立てないのでsqlxがsqlx-data.jsonを使うようにする
            // 環境変数にDATABASE_URLがあるとそれを見に行ってしまうのでDATABASE_URLがない.envファイルを読み込む
            from_filename(".ci.env").ok();
        }
        None => {
            from_filename(".env").ok();
        }
    }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .merge(graphql_schema::router(&pool));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
