use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://world:world123@localhost:7777/world-db")
        .await?;

    let x = sqlx::query!(r#"select * from public.city limit 10;"#)
        .fetch_all(&pool)
        .await?;

    println!("{:?}", x);

    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
