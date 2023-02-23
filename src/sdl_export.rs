use async_graphql::*;
use axum_graphql_example::graphql_schema::Query;

fn main() {
    let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription).finish();
    println!("{}", &schema.sdl());
}
