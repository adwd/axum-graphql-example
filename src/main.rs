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

    Ok(())
}
