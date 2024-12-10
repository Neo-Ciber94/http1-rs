use std::sync::{atomic::AtomicU64, Arc, Mutex};

use http1::server::Server;
use http1_web::{
    app::App, forms::form::Form, html::Html, json::Json, path::Path, redirect::Redirect,
    state::State, IntoResponse,
};
use serde::impl_serde_struct;

fn main() -> std::io::Result<()> {
    let app = App::new()
        .state(AppState::default())
        .get("/api/todos", get_todos)
        .post("/api/todos", create_todo)
        .post("/api/todos/:id", toggle_todo)
        .get("/", todos_index);

    Server::new()
        .on_ready(|addr| println!("Listening on http://localhost:{}", addr.port()))
        .listen("0.0.0.0:3000", app)
}

#[derive(Default, Clone)]
struct AppState {
    todos: Arc<Mutex<Vec<Todo>>>,
}

#[derive(Debug, Clone)]
struct Todo {
    id: u64,
    description: String,
    done: bool,
}

impl_serde_struct!(Todo =>  {
    id: u64,
    description: String,
    done: bool,
});

struct TodoInput {
    description: String,
}

impl_serde_struct!(TodoInput => {
    description: String
});

fn get_todos(State(state): State<AppState>) -> impl IntoResponse {
    let todos = state.todos.lock().unwrap().clone();
    Json(todos)
}

fn toggle_todo(State(state): State<AppState>, Path(todo_id): Path<u64>) -> impl IntoResponse {
    let mut todos = state.todos.lock().unwrap();

    if let Some(todo) = todos.iter_mut().find(|t| t.id == todo_id) {
        todo.done = !todo.done;
    }

    Redirect::see_other("/")
}

fn create_todo(
    State(state): State<AppState>,
    Form(TodoInput { description }): Form<TodoInput>,
) -> impl IntoResponse {
    let mut todos = state.todos.lock().unwrap();

    todos.push(Todo {
        id: next_id(),
        description,
        done: false,
    });

    Redirect::see_other("/")
}

fn todos_index(State(state): State<AppState>) -> impl IntoResponse {
    let todos_html = state
        .todos
        .lock()
        .unwrap()
        .iter()
        .map(|todo| {
            let Todo {
                description,
                done,
                id,
            } = todo;

            let is_done = if *done { "completed" } else { "pending" };
            let class = if *done { "todo-done" } else { "" };

            format!(
                "<li>
                    <span class={class}>{description}</span>
                    <form method=\"post\" action=\"/api/todos/{id}\">
                        <button>{is_done}</button>
                    </form>
                </li>"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Html(format!(
        r#"
        <html>
        <head>
            <title>Todos 📝</title>
            <meta charset="utf-8" />
            <link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' width='16' height='16'><text x='0' y='14' font-size='16'>📃</text></svg>">
            <style>
                body {{
                    background: linear-gradient(45deg, #f5f5dc, #e1dbb9); 
                    background-image: url('https://www.transparenttextures.com/patterns/old-wall.png'); 
                    background-repeat: repeat;
                }}

                .todo-done {{
                    opacity: 0.5;
                    text-decoration-line: line-through;
                }}
            </style>
        </head>
        <body>
            <h2>New Todo</h2>
            <form method="post" action="/api/todos">
                <input name="description" placeholder="Description" minlength="2" required/>
                <hr/>
                <button>Create</button>
            </form>
            <h2>Todos 📝</h2>
            <ol>{todos_html}</ol>
        </body>
        </html>
    "#,
    ))
}

fn next_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
}
