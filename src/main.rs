use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;
use sqlx::types::Uuid;
use sqlx::PgPool;
use webz::configuration::get_configuration;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let listener =
        tokio::net::TcpListener::bind(format!("0.0.0.0:{}", configuration.application_port))
            .await
            .unwrap();
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Connect to Postgres");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .with_state(Arc::new(connection_pool));

    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(
    State(connection_pool): State<Arc<PgPool>>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    let user_res = sqlx::query!("SELECT id, username FROM users",)
        .fetch_one(connection_pool.as_ref())
        .await;
    if user_res.is_ok() {
        return (StatusCode::CONFLICT, Json(user));
    }
    let res = sqlx::query!(
        r#"
        INSERT INTO users (id, username, password, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        user.id as i32,
        user.username,
        Utc::now()
    )
    .execute(connection_pool.as_ref())
    .await;
    if res.is_ok() {
        (StatusCode::CREATED, Json(user))
    } else {
        (StatusCode::BAD_REQUEST, Json(user))
    }
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
    password: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
