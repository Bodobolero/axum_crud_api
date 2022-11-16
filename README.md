# axum_crud_api
Simple example to learn creating CRUD rest apis in Rust with axum, sqlx with sqlite and utoipa (swagger) - without auth

Also shows how to run simple end-2-end tests with a stateful database for all rest verbs (GET, POST, PUT, DELETE) including testcases for error codes (not found).

Notes:
- while axum and sqlx potentially can be completely pure rust and only use safe code, the combination with sqlite (library written in C) is not pure Rust and uses unsafe code. 
- as far as I know sqlx with sqlite serializes all writers (even with connection pool). For production/better scalability one may consider using Postgres extension for sqlx instead.

## Used sources/credits: 

Carlos Marcano's Blog

- https://carlosmv.hashnode.dev/creating-a-rest-api-with-axum-sqlx-rust

Utoipa axum example

- https://github.com/juhaku/utoipa/blob/master/examples/todo-axum/src/main.rs

End-to-End-testing

- https://blog.logrocket.com/end-to-end-testing-for-rust-web-services/

