use async_graphql::{
    dataloader::{DataLoader, Loader},
    futures_util::TryFutureExt,
    *,
};
use log::debug;
use sqlx::{Pool, Postgres};
use std::{collections::HashMap, sync::Arc};

use crate::city::{City, CityLoader};

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
    /// Cities in the country
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

#[derive(derive_more::Constructor)]
pub struct CountryFlagLoader {
    pool: sqlx::Pool<Postgres>,
}

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

#[derive(Default)]
pub struct CountryQuery;

#[Object]
impl CountryQuery {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    /// Fetch countries
    async fn countries(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "search query by country name")] search: Option<String>,
        #[graphql(desc = "limit", default = 20, validator(maximum = 100))] limit: i64,
    ) -> Result<Vec<Country>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();

        let countries = sqlx::query_as!(
            Country,
            "select code, name, code2 from public.country where lower(name) like $1 limit $2;",
            search.map_or_else(|| "%%".into(), |s| format!("%{}%", s.to_lowercase())),
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(countries)
    }
}
