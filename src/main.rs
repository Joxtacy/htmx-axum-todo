use std::{env, sync::Arc, thread, time};

use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
    routing::{delete, get, post, put},
    Form, Router,
};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

use crate::templates::*;

mod templates;

const ARTIFICIAL_DELAY: time::Duration = time::Duration::from_millis(500);

#[derive(Debug, Clone)]
struct AppState {
    pool: sqlx::PgPool,
}

#[derive(Clone, Debug, Deserialize)]
struct Todo {
    title: String,
    id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
struct DbRow {
    id: Uuid,
    title: String,
    created_at: chrono::DateTime<chrono::Utc>,
    done: bool,
    done_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load `.env` file");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in `.env`");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url.as_str())
        .await
        .unwrap();

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let shared_state = Arc::new(AppState { pool: pool.clone() });

    let app = Router::new()
        .nest_service("/", ServeFile::new("assets/index.html"))
        .route("/todo", post(add_todo))
        .route("/todo", get(get_todos))
        .route("/todo/:id", get(get_todo))
        .route("/todo/:id", put(update_todo))
        .route("/todo/:id", delete(delete_todo))
        .route("/todo/:id/edit", get(get_edit_todo))
        .route("/todo/:id/done", put(done_todo))
        .route("/todo/title", post(validate_todo))
        .route("/hello", get(|| async { "hello wurl" }))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(shared_state);

    axum::Server::bind(&"0.0.0.0:3030".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct AddTodoData {
    todo: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTodoData {
    title: String,
    id: Uuid,
    done: bool,
}

async fn get_todos(State(state): State<Arc<AppState>>) -> Html<String> {
    println!("Get todos");
    // thread::sleep(ARTIFICIAL_DELAY);

    let todos = sqlx::query_as!(DbRow, "SELECT * FROM todos ORDER BY created_at ASC")
        .fetch_all(&state.pool)
        .await
        .unwrap();

    let todos = todos
        .into_iter()
        .map(|todo| {
            TodoItem {
                title: &todo.title,
                id: todo.id,
                done: todo.done,
            }
            .render()
            .unwrap()
        })
        .collect::<Vec<String>>()
        .join("\n")
        .clone();
    Html(todos)
}

async fn get_todo(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Html<String> {
    let todo = sqlx::query_as!(DbRow, "SELECT * FROM todos WHERE id = $1", id)
        .fetch_one(&state.pool)
        .await
        .unwrap();

    println!("Get todo: {}", todo.title);

    let todo_item = TodoItem {
        title: &todo.title,
        id,
        done: todo.done,
    };

    Html(todo_item.render().unwrap())
}

async fn get_edit_todo(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Html<String> {
    println!("Get edit todo: {}", id);
    let todo = sqlx::query_as!(DbRow, "SELECT * FROM todos WHERE id = $1", id)
        .fetch_one(&state.pool)
        .await
        .unwrap();

    let edit_todo = EditTodoItem {
        title: &todo.title,
        id,
        done: todo.done,
    };

    Html(edit_todo.render().unwrap())
}

async fn add_todo(
    State(state): State<Arc<AppState>>,
    Form(form): Form<AddTodoData>,
) -> Html<String> {
    println!("Adding todo: {}", form.todo);

    if form.todo.is_empty() {
        Html(
            ErrorModal {
                error_message: "Todo cannot be empty",
            }
            .render()
            .unwrap(),
        )
    } else {
        let id = Uuid::new_v4();

        sqlx::query!("INSERT INTO todos (title) VALUES ($1)", &form.todo)
            .execute(&state.pool)
            .await
            .unwrap();

        let new_todo = TodoItem {
            title: &form.todo,
            id,
            done: false,
        };
        let success_modal = SuccessModal {
            message: "Todo added",
        };
        Html(format!(
            "{}{}",
            new_todo.render().unwrap(),
            success_modal.render().unwrap()
        ))
    }
}

async fn delete_todo(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Html<String> {
    println!("Deleting todo: {}", id);

    sqlx::query!("DELETE FROM todos WHERE id = $1", id)
        .execute(&state.pool)
        .await
        .unwrap();

    Html("".to_string()) // deleted so we return no html
}

async fn update_todo(
    State(state): State<Arc<AppState>>,
    Form(form): Form<UpdateTodoData>,
) -> Html<String> {
    println!("Updating todo: {}", form.title);

    sqlx::query!(
        "UPDATE todos SET title = $1 WHERE id = $2",
        &form.title,
        &form.id
    )
    .execute(&state.pool)
    .await
    .unwrap();

    let new_todo = TodoItem {
        title: &form.title,
        id: form.id,
        done: form.done,
    };

    Html(new_todo.render().unwrap())
}

async fn done_todo(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> Html<String> {
    println!("Done todo id: {}", id);
    let _rows_affected = sqlx::query!(
        "UPDATE todos SET done = NOT done, done_at = CASE WHEN done THEN NULL ELSE now() END WHERE id = $1",
        id
    )
    .execute(&state.pool)
    .await
    .unwrap()
    .rows_affected();

    let todo = sqlx::query_as!(DbRow, "SELECT * FROM todos WHERE id = $1", id)
        .fetch_one(&state.pool)
        .await
        .unwrap();

    let new_todo = TodoItem {
        title: todo.title.as_str(),
        id,
        done: todo.done,
    };

    Html(new_todo.render().unwrap())
}

async fn validate_todo(Form(form): Form<AddTodoData>) -> Html<String> {
    println!("Validating todo: {}", form.todo);
    // thread::sleep(ARTIFICIAL_DELAY);

    if form.todo.is_empty() {
        Html(
            ValidateTodoModal {
                error_message: "Todo cannot be empty",
                value: "",
            }
            .render()
            .unwrap(),
        )
    } else {
        Html(
            ValidateTodoModal {
                error_message: "",
                value: &form.todo,
            }
            .render()
            .unwrap(),
        )
    }
}
