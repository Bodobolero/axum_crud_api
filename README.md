# axum_crud_api
Simple example to learn creating CRUD rest apis in Rust with axum, sqlx with sqlite and utoipa (swagger) - without auth

Notes:
- while axum and sqlx potentially can be completely pure rust and only use safe code, the combination with sqlite (library written in C) is not pure Rust and uses unsafe code. 
- as far as I know sqlx with sqlite serializes all writers (even with connection pool). For production/better scalability one may consider using Postgres extension for sqlx instead.

## Used sources/credits: 

Carlos Marcano's Blog

- https://carlosmv.hashnode.dev/creating-a-rest-api-with-axum-sqlx-rust

Utoipa axum example

- https://github.com/juhaku/utoipa/blob/master/examples/todo-axum/src/main.rs

