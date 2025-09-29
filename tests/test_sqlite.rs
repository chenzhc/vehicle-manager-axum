#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::{any::Any, env};
use log::info;
use sqlx::{migrate::MigrateDatabase, postgres::PgPoolOptions, FromRow, Sqlite, SqlitePool};
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


#[tokio::test]
async fn it_pg_batch_execute_test01() -> anyhow::Result<()> {
    init();
     // 获取数据库连接 url
    let database_url = env::var("PG_DATABASE_URL").expect("PG_DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool).await?;
    info!("{:?}", row);

    let rs = sqlx::query(
        " CREATE TABLE IF NOT EXISTS book  (
            id              SERIAL PRIMARY KEY,
            title           VARCHAR NOT NULL,
            author_id       INTEGER NOT NULL REFERENCES author
            )"
    )
    .execute(&pool)
    .await?;
    info!("{:?}", rs);

    Ok(())
}