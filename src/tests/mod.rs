use hyper::{body::to_bytes, client::HttpConnector, Body, Client as HyperClient, Method, Request};
use hyper_tls::HttpsConnector;
use mock::*;
use sqlx::Connection;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    ConnectOptions,
};
use std::str::FromStr;
use std::sync::RwLock;

mod mock;

const TEST_HOST: &str = "http://127.0.0.1:3000";
const POST_TASK_URI: &str = "/tasks";
const GET_TASKS_URI: &str = "/tasks";

lazy_static! {
    static ref SERVER: RwLock<Server> = RwLock::new(Server::new());
}

async fn delete_all_tasks() -> anyhow::Result<()> {
    // tabula rasa for reentrant tests
    let mut conn = SqliteConnectOptions::from_str(&crate::DATABASE_URL)?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true)
        .connect()
        .await?;
    sqlx::query("DELETE FROM task").execute(&mut conn).await?;
    conn.close().await?;
    Ok(())
}

async fn init_real_server() -> anyhow::Result<()> {
    delete_all_tasks().await?;
    SERVER.write().unwrap().init_server().await;
    Ok(())
}

fn http_client() -> HyperClient<HttpsConnector<HttpConnector>> {
    let https = HttpsConnector::new();
    HyperClient::builder().build::<_, Body>(https)
}

#[tokio::test]
async fn test_create_one_task_and_list_tasks_e2e() -> anyhow::Result<()> {
    init_real_server().await?;
    let http_client = http_client();

    let req = Request::builder()
        .method(Method::POST)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .uri(TEST_HOST.to_string() + POST_TASK_URI)
        .body(Body::from(r#"{"task":"my first test task"}"#))
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 201);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, r#"{"id":1,"task":"my first test task"}"#);

    let req = Request::builder()
        .method(Method::GET)
        .uri(TEST_HOST.to_string() + GET_TASKS_URI)
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, r#"[{"id":1,"task":"my first test task"}]"#);
    Ok(())
}

#[tokio::test]
async fn test_create_two_tasks_and_list_tasks_e2e() -> anyhow::Result<()> {
    init_real_server().await?;
    let http_client = http_client();

    let req = Request::builder()
        .method(Method::POST)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .uri(TEST_HOST.to_string() + POST_TASK_URI)
        .body(Body::from(r#"{"task":"my first test task"}"#))
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 201);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, r#"{"id":1,"task":"my first test task"}"#);

    let req = Request::builder()
        .method(Method::POST)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .uri(TEST_HOST.to_string() + POST_TASK_URI)
        .body(Body::from(r#"{"task":"my second test task"}"#))
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 201);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, r#"{"id":2,"task":"my second test task"}"#);

    let req = Request::builder()
        .method(Method::GET)
        .uri(TEST_HOST.to_string() + GET_TASKS_URI)
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(
        body_bytes,
        r#"[{"id":1,"task":"my first test task"},{"id":2,"task":"my second test task"}]"#
    );
    Ok(())
}

#[tokio::test]
async fn test_list_empty_tasks_e2e() -> anyhow::Result<()> {
    init_real_server().await?;
    let http_client = http_client();
    let req = Request::builder()
        .method(Method::GET)
        .uri(TEST_HOST.to_string() + GET_TASKS_URI)
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, r#"[]"#);
    Ok(())
}
