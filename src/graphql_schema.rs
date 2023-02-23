use async_graphql::{dataloader::DataLoader, http::GraphiQLSource, *};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use sqlx::{Pool, Postgres};

use crate::{
    city::{CityLoader, CityQuery},
    country::{CountryFlagLoader, CountryQuery},
};

#[derive(MergedObject, Default)]
pub struct Query(CountryQuery, CityQuery);

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
    let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription)
        .data(pool.clone())
        .data(DataLoader::new(
            CityLoader::new(pool.clone()),
            tokio::task::spawn,
        ))
        .data(DataLoader::new(
            CountryFlagLoader::new(pool.clone()),
            tokio::task::spawn,
        ))
        .finish();

    println!("GraphiQL IDE: http://localhost:3000/graphql");

    Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
}
