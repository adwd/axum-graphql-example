use std::{collections::HashMap, sync::Arc};

use async_graphql::{connection::*, dataloader::Loader, *};
use log::debug;
use sqlx::{Pool, Postgres};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
/// City
pub struct City {
    #[graphql(skip)]
    pub row_id: i32,
    pub name: String,
    #[graphql(skip)]
    pub country_code: String,
}

#[ComplexObject]
impl City {
    async fn id(&self) -> ID {
        self.row_id.to_string().into()
    }
}

#[derive(derive_more::Constructor)]
pub struct CityLoader {
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
            "select id as row_id, name, country_code from public.city where country_code = any($1)",
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

#[derive(Default)]
pub struct CityQuery;

#[Object]
impl CityQuery {
    async fn cities(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<Connection<i32, City, EmptyFields, EmptyFields>> {
        // https://async-graphql.github.io/async-graphql/en/cursor_connections.html
        // https://relay.dev/graphql/connections.htm
        query(
            after,
            before,
            first,
            last,
            |after, _before, first, _last| async move {
                let pool = ctx.data_unchecked::<Pool<Postgres>>();

                let cities = if let Some(a) = after {
                    // i32の変数に束縛し直すとかやらないとコンパイルエラーになってしまう
                    let a: i32 = a;
                    sqlx::query_as!(
                        City,
                        "select id as row_id, name, country_code from public.city where id > $1 order by id asc limit $2;",
                        a,
                        first.map_or(20, |v| (v as i64))
                    )
                    .fetch_all(pool)
                    .await?
                } else {
                    sqlx::query_as!(
                        City,
                        "select id as row_id, name, country_code from public.city order by id asc limit $1;",
                        first.map_or(20, |v| (v as i64))
                    )
                    .fetch_all(pool)
                    .await?
                };

                // todo: has_previous, has_nextをちゃんとやる
                let mut connection = Connection::new(true, true);
                connection.edges.extend(
                    cities
                        .into_iter()
                        .map(|c| Edge::with_additional_fields(c.row_id, c, EmptyFields)),
                );
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}
