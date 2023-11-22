use std::{
    sync::{Arc, Mutex},
    thread, time,
};

use axum::{
    extract::State,
    response::Html,
    routing::{delete, get, post, put},
    Form, Router,
};
use serde::Deserialize;
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

#[derive(Debug)]
struct AppState {
    todos: Vec<Todo>,
}

#[derive(Debug, Deserialize)]
struct Todo {
    title: String,
    id: Uuid,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(AppState { todos: vec![] }));

    let app = Router::new()
        .nest_service("/", ServeFile::new("assets/index.html"))
        .route("/todo", post(add_todo))
        .route("/todo", get(get_todos))
        .route("/todo", put(update_todo))
        .route("/todo", delete(delete_todo))
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
struct DeleteTodoData {
    id: Uuid,
}

#[derive(Debug, Deserialize)]
struct UpdateTodoData {
    title: String,
    id: Uuid,
}

async fn get_todos(State(state): State<Arc<Mutex<AppState>>>) -> Html<String> {
    println!("Get todos");
    let two_sec = time::Duration::from_secs(2);
    thread::sleep(two_sec);
    let state = state.lock().unwrap();
    let todos = &state.todos;
    let todos = todos
        .into_iter()
        .map(|todo| {
            format!(
                "<li _=\"on click log \'Clicked on: {}\'\">{}</li>",
                todo.title, todo.title
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
        .clone();
    Html(todos)
}

async fn add_todo(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(form): Form<AddTodoData>,
) -> Html<String> {
    println!("Adding todo: {}", form.todo);

    if form.todo.is_empty() {
        Html("<div id='error-modal' hx-swap-oob='true' class='error-message'>Todo cannot be empty</div>".to_string())
    } else {
        let id = Uuid::new_v4();

        let new_todo = Todo {
            title: form.todo.clone(),
            id,
        };
        state.lock().unwrap().todos.push(new_todo);
        Html(format!(
            "<li>{}</li>
             <div id='success-modal' hx-swap-oob='true'>Todo added</div>",
            form.todo
        ))
    }
}

async fn delete_todo(State(state): State<Arc<Mutex<AppState>>>, Form(form): Form<DeleteTodoData>) {
    println!("Deleting todo: {}", form.id);
    state
        .lock()
        .unwrap()
        .todos
        .retain(|todo| todo.id != form.id);
}

async fn update_todo(State(_state): State<Arc<Mutex<AppState>>>, Form(form): Form<UpdateTodoData>) {
    println!("Updating todo: {}, {}", form.id, form.title);

    /*
    let new_title = form.title.clone();
    state
        .lock()
        .unwrap()
        .todos
        .into_iter()
        .for_each(|mut todo| {
            if todo.id == form.id {
                todo.title = new_title;
            }
        });
    */
}

async fn validate_todo(Form(form): Form<AddTodoData>) -> Html<String> {
    println!("Validating todo: {}", form.todo);
    let two_sec = time::Duration::from_secs(2);
    thread::sleep(two_sec);

    if form.todo.is_empty() {
        Html(
            r###"
<div hx-target="this" hx-swap="outerHTML" class="error">
  <input type="text" name="todo" hx-post="/todo/title" hx-indicator="#ind" value="" hx-disabled-elt="this" hx-sync="closest form:drop">
  <img id="ind" src="/assets/oval_loader.svg" class="htmx-indicator"/>
  <div class='error-message'>Todo cannot be empty</div>
</div>"###
                .to_string(),
        )
    } else {
        Html(
            format!(
                r###"
<div hx-target="this" hx-swap="outerHTML" class="error">
  <input type="text" name="todo" hx-post="/todo/title" hx-indicator="#ind" value="{}" hx-disabled-elt="this" hx-sync="closest form:drop">
  <img id="ind" src="/assets/oval_loader.svg" class="htmx-indicator"/>
</div>"###,
                form.todo
            )
            .to_string(),
        )
    }
}
