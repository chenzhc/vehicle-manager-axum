#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::any::Any;

use log::info;
use sqlx::{migrate::MigrateDatabase, FromRow, Sqlite, SqlitePool};
use vehicle_manager_axum::init;

const DB_URL: &str = "sqlite://sqlite.db";

#[tokio::test]
async fn it_sqlit_test01() -> anyhow::Result<()> {
    init();

    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        info!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => info!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        info!("Database already exists!");
    }

    let db = SqlitePool::connect(DB_URL).await.unwrap();

    let result = sqlx::query("CREATE TABLE IF NOT EXISTS users (id integer primary key not null, 
        name varchar(250) not null); ")
        .execute(&db)
        .await
        .unwrap();
    info!("Create user table result: {:?}", result);

    Ok(())
}

#[derive(Clone, FromRow, Debug)]
struct User {
    id: i64,
    name: String,
}

#[tokio::test]
async fn it_query_sqlite_test01() -> anyhow::Result<()> {
    init();
    let db = SqlitePool::connect(DB_URL).await.unwrap();

    let result = sqlx::query(
        "SELECT name
         FROM sqlite_schema
         WHERE type ='table' 
         AND name NOT LIKE 'sqlite_%';",
    )
    .fetch_all(&db)
    .await
    .unwrap();
    for (idx, row) in result.iter().enumerate() {
        info!("[{}]", idx);
    }

    let result = sqlx::query("INSERT INTO users (name) VALUES (?)")
        .bind("bobby")
        .execute(&db)
        .await
        .unwrap();
    info!("Query result: {:?}", result);
    let user_results = sqlx::query_as::<_, User>("SELECT id, name FROM users")
        .fetch_all(&db)
        .await
        .unwrap();
    for user in user_results {
        info!("[{}] name: {}", user.id, &user.name);
    }

    Ok(())
}