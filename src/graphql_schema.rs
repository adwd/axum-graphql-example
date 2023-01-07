use async_graphql::{http::GraphiQLSource, *};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use sqlx::{Pool, Postgres};

#[derive(SimpleObject, Debug)]
pub struct Country {
    pub code: String,
    pub name: String,
}

pub struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    async fn countries(&self, ctx: &Context<'_>) -> Result<Vec<Country>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let countries = sqlx::query_as!(Country, "select code, name from public.country limit 10;")
            .fetch_all(pool)
            .await?;

        Ok(countries)
    }
}

async fn graphql_handler(
    schema: Extension<Schema<Query, EmptyMutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn router(pool: &Pool<Postgres>) -> Router {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(pool.clone())
        .finish();

    println!("GraphiQL IDE: http://localhost:3000/graphql");

    Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
}
