#[cfg(test)]
mod integration_tests {

    use std::time::Duration;

    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
        routing::get,
        Router,
    };
    use axum_graphql_example::graphql_schema;
    use serde_json::{json, Value};
    use sqlx::postgres::PgPoolOptions;
    use testcontainers::{clients, core::WaitFor, GenericImage};
    use tower::ServiceExt;

    extern crate axum_graphql_example;

    async fn get_app() -> Router {
        let docker = clients::Cli::default();
        // let world_server = docker.run("ghusta/postgres-world-db");
        let image = GenericImage::new("ghusta/postgres-world-db", "2.10")
            .with_exposed_port(5432)
            .with_wait_for(WaitFor::message_on_stdout(
                "database system is ready to accept connections",
            ))
            // 少し待たないとテストがこけることがある
            .with_wait_for(WaitFor::Duration {
                length: Duration::from_secs(3),
            });
        let container = docker.run(image);
        container.start();

        let port = container.get_host_port_ipv4(5432);

        println!("PostgreSQL World DB: {}", port);

        let database_url = format!("postgres://world:world123@localhost:{}/world-db", port);
        println!("database_url: {}", database_url);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap();

        println!("pool: {:?}", pool);

        Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .merge(graphql_schema::router(&pool))
    }

    #[tokio::test]
    #[ignore]
    async fn hello_world() {
        let app = get_app().await;
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(std::str::from_utf8(&body).unwrap(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_graphiql() {
        let app = get_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/graphql")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        assert!(
            response.headers().iter().any(|(k, v)| {
                k == http::header::CONTENT_TYPE && v == "text/html; charset=utf-8"
            }),
            "Header must include Content-Type: text/html {:?}",
            response.headers()
        );
    }

    #[tokio::test]
    #[ignore]
    async fn cities() {
        println!("cities");
        let app = get_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/graphql")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(
                        r#"{ "query":"{ cities(first: 3) { edges { node { id name } } } }" }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        println!("response: {:?}", response);

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        println!("body: {:?}", body);
        assert_eq!(
            body["data"]["cities"]["edges"],
            json!([
              {
                "node": {
                  "id": "1",
                  "name": "Kabul"
                }
              },
              {
                "node": {
                  "id": "2",
                  "name": "Qandahar"
                }
              },
              {
                "node": {
                  "id": "3",
                  "name": "Herat"
                }
              }
            ])
        );
        println!("cities done");
    }
}
