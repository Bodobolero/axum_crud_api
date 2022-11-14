use hyper::{body::to_bytes, client::HttpConnector, Body, Client as HyperClient, Method, Request};
use hyper_tls::HttpsConnector;
use mock::*;
use sqlx::Connection;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    ConnectOptions,
};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, OwnedMutexGuard};

mod mock;

const TEST_HOST: &str = "http://127.0.0.1:3000";
const POST_TASK_URI: &str = "/tasks";
const GET_TASKS_URI: &str = "/tasks";
const DELETE_TASK_URI: &str = "/tasks/";
const PUT_TASK_URI: &str = "/tasks/";
const GET_TASK_URI: &str = "/tasks/";

// we use a single instance of Server which has the shared state in the sqlite database
// and we need to make sure that only one testcase locks this resource
// as long as the testcase runs - otherwise the results of list tasks and the
// assigned ids (primary key in sqlite) will not be reliable and the tests may fail
// so each testcase has to use
// let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
// init_and_lock_real_server(&mut locked_server).await?;
lazy_static! {
    static ref SERVER: Arc<Mutex<Server>> = Arc::new(Mutex::new(Server::new()));
}

/**
 * Create a defined state in the database by deleting all tasks.
 * Here we rely on sqlite delete truncate optimization - which
 * also resets the primary key id.
 * Note: because all testcases share the same database we cannot
 * run the tests in parallel.
 */
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

async fn init_and_lock_real_server(server: &mut OwnedMutexGuard<Server>) -> anyhow::Result<()> {
    server.init_server().await;
    delete_all_tasks().await?;
    Ok(())
}

fn http_client() -> HyperClient<HttpsConnector<HttpConnector>> {
    let https = HttpsConnector::new();
    HyperClient::builder().build::<_, Body>(https)
}

#[tokio::test]
async fn test_create_one_task_and_list_tasks_e2e() -> anyhow::Result<()> {
    let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
    init_and_lock_real_server(&mut locked_server).await?;
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
async fn test_create_one_task_and_get_task_e2e() -> anyhow::Result<()> {
    let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
    init_and_lock_real_server(&mut locked_server).await?;
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
        .uri(TEST_HOST.to_string() + GET_TASK_URI + "1")
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, r#"{"id":1,"task":"my first test task"}"#);
    Ok(())
}

#[tokio::test]
async fn test_create_two_tasks_and_list_tasks_e2e() -> anyhow::Result<()> {
    let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
    init_and_lock_real_server(&mut locked_server).await?;
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
    let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
    init_and_lock_real_server(&mut locked_server).await?;
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

#[tokio::test]
async fn test_create_and_delete_one_task_and_list_tasks_e2e() -> anyhow::Result<()> {
    let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
    init_and_lock_real_server(&mut locked_server).await?;
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
        .method(Method::DELETE)
        .uri(TEST_HOST.to_string() + DELETE_TASK_URI + "1")
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);

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

#[tokio::test]
async fn test_create_and_update_one_task_and_list_tasks_e2e() -> anyhow::Result<()> {
    let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
    init_and_lock_real_server(&mut locked_server).await?;
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
        .method(Method::PUT)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .uri(TEST_HOST.to_string() + PUT_TASK_URI + "1")
        .body(Body::from(r#"{"task":"my first updated test task"}"#))
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, r#"{"task":"my first updated test task"}"#);

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
        r#"[{"id":1,"task":"my first updated test task"}]"#
    );
    Ok(())
}

#[tokio::test]
async fn test_get_wrong_task_e2e() -> anyhow::Result<()> {
    let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
    init_and_lock_real_server(&mut locked_server).await?;
    let http_client = http_client();
    let req = Request::builder()
        .method(Method::GET)
        .uri(TEST_HOST.to_string() + GET_TASK_URI + "4711")
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 404);
    Ok(())
}

#[tokio::test]
async fn test_delete_wrong_task_e2e() -> anyhow::Result<()> {
    let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
    init_and_lock_real_server(&mut locked_server).await?;
    let http_client = http_client();
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(TEST_HOST.to_string() + DELETE_TASK_URI + "4711")
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 404);
    Ok(())
}

#[tokio::test]
async fn test_update_wrong_task_e2e() -> anyhow::Result<()> {
    let mut locked_server: OwnedMutexGuard<Server> = SERVER.clone().lock_owned().await;
    init_and_lock_real_server(&mut locked_server).await?;
    let http_client = http_client();
    let req = Request::builder()
        .method(Method::PUT)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .uri(TEST_HOST.to_string() + PUT_TASK_URI + "4711")
        .body(Body::from(r#"{"task":"my first updated test task"}"#))
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 404);
    Ok(())
}
