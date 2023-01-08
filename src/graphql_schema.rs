use async_graphql::{
    dataloader::{DataLoader, Loader},
    futures_util::TryFutureExt,
    http::GraphiQLSource,
    *,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use log::debug;
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};

#[derive(SimpleObject, Debug)]
#[graphql(complex)]
pub struct Country {
    /// Code of Country
    pub code: String,
    /// Name of Country
    pub name: String,
    #[graphql(skip)]
    pub code2: String,
}

#[ComplexObject]
impl Country {
    /// Cities within country
    async fn cities(&self, ctx: &Context<'_>) -> Result<Vec<City>> {
        ctx.data_unchecked::<DataLoader<CityLoader>>()
            .load_one(self.code.clone())
            .map_ok(|x| x.unwrap_or_default())
            // TODO: ちゃんとしたエラーハンドリングを考える
            .map_err(|e| e.into())
            .await
    }

    /// Emoji of country flag
    async fn flag(&self, ctx: &Context<'_>) -> Result<Option<String>> {
        ctx.data_unchecked::<DataLoader<CountryFlagLoader>>()
            .load_one(self.code2.clone())
            // TODO: ちゃんとしたエラーハンドリングを考える
            .map_err(|e| e.into())
            .await
    }
}

#[derive(SimpleObject, Debug, Clone)]
/// City
pub struct City {
    pub name: String,
    #[graphql(skip)]
    pub country_code: String,
}

struct CityLoader {
    pool: sqlx::Pool<Postgres>,
}

#[async_trait::async_trait]
impl Loader<String> for CityLoader {
    type Value = Vec<City>;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        debug!("DataLoader for city, keys: {:?}", keys);

        // https://github.com/launchbadge/sqlx/blob/main/FAQ.md#how-can-i-do-a-select--where-foo-in--query
        let cities = sqlx::query_as!(
            City,
            "select name, country_code from public.city where country_code = any($1)",
            &keys[..]
        )
        .fetch_all(&self.pool)
        .await?;

        let mut cities_map: HashMap<String, Vec<City>> = HashMap::new();
        for c in cities.into_iter() {
            if let Some(cities) = cities_map.get_mut(&c.country_code) {
                cities.push(c);
            } else {
                cities_map.insert(c.country_code.clone(), vec![c]);
            }
        }

        Ok(cities_map)
    }
}

struct CountryFlagLoader {
    pool: sqlx::Pool<Postgres>,
}

#[async_trait::async_trait]
impl Loader<String> for CountryFlagLoader {
    type Value = String;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        debug!("DataLoader for country flag, keys: {:?}", keys);

        let records = sqlx::query!(
            "select code2, emoji from public.country_flag where code2 =  any($1)",
            &keys[..]
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(HashMap::from_iter(
            records.into_iter().map(|r| (r.code2, r.emoji)),
        ))
    }
}

pub struct Query;

#[Object]
impl Query {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    /// Fetch countries
    async fn countries(&self, ctx: &Context<'_>) -> Result<Vec<Country>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let countries = sqlx::query_as!(
            Country,
            "select code, name, code2 from public.country limit 10;"
        )
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
        .data(DataLoader::new(
            CityLoader { pool: pool.clone() },
            tokio::task::spawn,
        ))
        .data(DataLoader::new(
            CountryFlagLoader { pool: pool.clone() },
            tokio::task::spawn,
        ))
        .finish();

    println!("GraphiQL IDE: http://localhost:3000/graphql");

    Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
}
