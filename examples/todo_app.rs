use db::DB;
use http1::server::Server;
use http1_web::{app::App, middleware::{logging::Logging, redirection::Redirection}};
use routes::{api_routes, home_routes, todos_routes};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let addr = "127.0.0.1:5000".parse().unwrap();

    let app = App::new()
        .state(DB::new())
        .middleware(Logging)
        //.middleware(Redirection::new("/", "/login"))
        .scope("/api", api_routes())
        .scope("/", home_routes())
        .scope("/todos", todos_routes());

    Server::new(addr)
        .on_ready(|addr| log::info!("Listening on http://{addr}"))
        .start(app)
}

const COOKIE_SESSION_NAME: &str = "auth-session-id";

mod routes {
    use http1::{body::Body, response::Response};
    use http1_web::{
        app::Scope, cookies::{Cookie, Cookies}, error_response::{ErrorResponse, ErrorStatusCode}, forms::form::Form, from_request::FromRequestRef, html::{self, element::HTMLElement}, impl_serde_struct, into_response::IntoResponse, path::Path, redirect::Redirect, state::State
    };

    use crate::{
        components::Title,
        db::{Todo, User, DB},
        COOKIE_SESSION_NAME,
    };

    #[derive(Debug)]
    struct AuthenticatedUser(pub User);

    enum AuthenticatedUserRejection {
        StateError,
        RedirectToLogin
    }

    impl IntoResponse for AuthenticatedUserRejection {
        fn into_response(self) -> Response<Body> {
            match self {
                AuthenticatedUserRejection::StateError => ErrorStatusCode::InternalServerError.into_response(),
                AuthenticatedUserRejection::RedirectToLogin => Redirect::see_other("/login").into_response(),
            }
        }
    }

    impl FromRequestRef for AuthenticatedUser {
        type Rejection = AuthenticatedUserRejection;

        fn from_request_ref(
            req: &http1::request::Request<http1::body::Body>,
        ) -> Result<Self, Self::Rejection> {
            let State(db) = State::<DB>::from_request_ref(req)
            .inspect_err(|err| {
                log::error!("Failed to get database: {err}");
            })
                .map_err(|_| AuthenticatedUserRejection::StateError)?;

            let cookies = Cookies::from_request_ref(req).unwrap_or_default();

            let session_cookie = cookies
                .get(super::COOKIE_SESSION_NAME)
                .ok_or_else(|| AuthenticatedUserRejection::RedirectToLogin)?;

            match crate::db::get_session_user(&db, session_cookie.value().to_owned()) {
                Ok(Some(user)) => Ok(AuthenticatedUser(user)),
                x => {
                    log::error!("user session not found: {x:?}");
                    Err(AuthenticatedUserRejection::RedirectToLogin)
                },
            }
        }
    }

    pub fn api_routes() -> Scope {
        #[derive(Debug)]
        struct LoginUser {
            pub username: String,
        }

        impl_serde_struct!(LoginUser => {
             username: String
        });

        struct CreateTodo {
            pub title: String,
            pub description: Option<String>,
        }
    
        impl_serde_struct!(CreateTodo => {
             title: String,
             description: Option<String>,
        });

        struct UpdateTodo {
            pub id: u64,
            pub title: String,
            pub description: Option<String>,
        }
    
        impl_serde_struct!(UpdateTodo => {
             id: u64,
             title: String,
             description: Option<String>,
        });

        Scope::new()
        .post(
            "/login",
            |State(db): State<DB>,
             Form(input): Form<LoginUser>|
             -> Result<Response<Body>, ErrorResponse> {
                let user = crate::db::insert_user(&db, input.username)?;
                let session = crate::db::create_session(&db, user.id)?;

                log::info!("User created: {user:?}");

                Ok((
                    Redirect::see_other("/todos"),
                    Cookie::new(COOKIE_SESSION_NAME, session.id.clone())
                        .path("/")
                        .http_only(true)
                        .same_site(http1_web::cookies::SameSite::Strict)
                        .build(),
                )
                    .into_response())
            },
        )
        .post("/todos/create", |State(db): State<DB>, AuthenticatedUser(user): AuthenticatedUser, Form(todo): Form<CreateTodo>|-> Result<Redirect, ErrorResponse> {
            crate::db::insert_todo(&db, todo.title, todo.description.filter(|x| !x.is_empty()), user.id)?;
            
            Ok(Redirect::see_other("/todos"))
        })
        .post("/todos/update", |State(db): State<DB>,Form(form): Form<Todo>|-> Result<Redirect, ErrorResponse> {
            crate::db::update_todo(&db, form)?;
            
            Ok(Redirect::see_other("/todos"))
        })
        .post("/todos/delete/:todo_id", |State(db): State<DB>, Path(todo_id): Path<u64>|-> Result<Redirect, ErrorResponse> {
            crate::db::delete_todo(&db, todo_id)?;
            
            Ok(Redirect::see_other("/todos"))
        })
        .post("/todos/toggle/:todo_id", |State(db): State<DB>, Path(todo_id): Path<u64>|-> Result<Redirect, ErrorResponse> {
            let mut todo = match crate::db::get_todo(&db, todo_id)? {
                Some(x) => x,
                None => return Err(ErrorStatusCode::NotFound.into()),
            };

            todo.is_done = !todo.is_done;
            crate::db::update_todo(&db, todo)?;
            
            Ok(Redirect::see_other("/todos"))
        })
    }

    pub fn todos_routes() -> Scope {
        Scope::new()
            .get("/", |State(db): State<DB>, AuthenticatedUser(user): AuthenticatedUser| -> Result<HTMLElement, ErrorResponse> {
                let todos = crate::db::get_all_todos(&db, user.id)?;
    
                dbg!(&todos);

                Ok(html::html(|| {
                    Title("TodoApp | Todos", ());
    
                    html::body(|| {
                        html::div(|| {
                            html::class("min-h-screen bg-gray-100 p-6");
    
                            html::h1(|| {
                                html::content("Your Todos");
                                html::class("text-3xl font-bold text-center text-blue-600 mb-6");
                            });
    
                            // Link to create a new todo
                            html::a(|| {
                                html::attr("href", "/todos/create");
                                html::content("Create New Todo");
                                html::class("mt-6 p-3 bg-green-500 text-white rounded hover:bg-green-600 transition inline-block");
                            });
                    
                            html::div(|| {
                                todos.iter().for_each(|todo| {
                                    html::div(|| {
                                        html::class("bg-white shadow-lg rounded p-4 mb-4");
    
                                        html::h2(|| {
                                            html::content(&todo.title);
                                            html::class("text-xl font-bold text-gray-800");
                                        });
    
                                        if let Some(desc) = &todo.description {
                                            html::p(|| {
                                                html::content(desc);
                                                html::class("text-gray-600 mt-2");
                                            });
                                        }
    
                                        // Toggle button
                                        html::form(|| {
                                            html::attr("method", "post");
                                            html::attr("action", format!("/api/todos/toggle/{}", todo.id));
    
                                            html::button(|| {
                                                html::content(if todo.is_done { "Mark as Incomplete" } else { "Mark as Complete" });
                                                html::class("mt-4 p-2 bg-yellow-500 text-white rounded hover:bg-yellow-600 transition");
                                            });
                                        });
    
                                        // Edit button
                                        html::a(|| {
                                            html::attr("href", format!("/todos/edit/{}", todo.id));
                                            html::content("Edit");
                                            html::class("mt-2 p-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition inline-block");
                                        });
    
                                        // Delete form
                                        html::form(|| {
                                            html::attr("method", "post");
                                            html::attr("action", format!("/api/todos/delete/{}", todo.id));
    
                                            html::button(|| {
                                                html::content("Delete");
                                                html::class("mt-2 p-2 bg-red-500 text-white rounded hover:bg-red-600 transition");
                                            });
                                        });
                                    });
                                });
                            });
    
         
                        });
                    });
                }))
            })
            .get("/create", | AuthenticatedUser(user): AuthenticatedUser| {
                html::html(|| {
                    Title("TodoApp | Create Todo", ());
    
                    html::body(|| {
                        html::div(|| {
                            html::class("min-h-screen bg-gray-100 p-6 flex items-center justify-center");
    
                            html::div(|| {
                                html::class("bg-white p-8 rounded shadow-lg w-full max-w-md");
    
                                html::h1(|| {
                                    html::content("Create New Todo");
                                    html::class("text-3xl font-bold text-center text-blue-600 mb-6");
                                });
    
                                html::form(|| {
                                    html::attr("method", "post");
                                    html::attr("action", "/api/todos/create");
    
                                    html::input(|| {
                                        html::attr("type", "text");
                                        html::attr("placeholder", "Title");
                                        html::attr("name", "title");
                                        html::class("mt-4 p-3 border border-gray-300 rounded w-full");
                                    });
    
                                    html::textarea(|| {
                                        html::attr("placeholder", "Description (optional)");
                                        html::attr("name", "description");
                                        html::class("mt-4 p-3 border border-gray-300 rounded w-full");
                                    });
    
                                    html::button(|| {
                                        html::content("Create Todo");
                                        html::class("mt-6 p-3 bg-green-500 text-white rounded w-full hover:bg-green-600 transition");
                                    });
                                });
                            });
                        });
                    });
                })
            })
            .get("/edit/:todo_id", |State(db): State<DB>, AuthenticatedUser(user): AuthenticatedUser, Path(todo_id): Path<u64>| -> Result<HTMLElement, ErrorResponse> {
                let todo = match crate::db::get_todo(&db, todo_id)? {
                    Some(x) => x,
                    None => {
                        return Err(ErrorStatusCode::NotFound.into())
                    }
                };
    
                Ok(html::html(|| {
                    Title("TodoApp | Edit Todo", ());
    
                    html::body(|| {
                        html::div(|| {
                            html::class("min-h-screen bg-gray-100 p-6 flex items-center justify-center");
    
                            html::div(|| {
                                html::class("bg-white p-8 rounded shadow-lg w-full max-w-md");
    
                                html::h1(|| {
                                    html::content("Edit Todo");
                                    html::class("text-3xl font-bold text-center text-blue-600 mb-6");
                                });
    
                                html::form(|| {
                                    html::attr("method", "post");
                                    html::attr("action", "/api/todos/update");
    
                                    html::input(|| {
                                        html::attr("type", "hidden");
                                        html::attr("name", "id");
                                        html::attr("value", todo.id);
                                    });
                                    
                                    html::input(|| {
                                        html::attr("type", "text");
                                        html::attr("placeholder", "Title");
                                        html::attr("name", "title");
                                        html::attr("value", &todo.title);
                                        html::class("mt-4 p-3 border border-gray-300 rounded w-full");
                                    });
    
                                    html::textarea(|| {
                                        html::attr("placeholder", "Description (optional)");
                                        html::attr("name", "description");
                                        html::content(todo.description.clone().unwrap_or_default());
                                        html::class("mt-4 p-3 border border-gray-300 rounded w-full");
                                    });
    
                                    html::button(|| {
                                        html::content("Update Todo");
                                        html::class("mt-6 p-3 bg-blue-500 text-white rounded w-full hover:bg-blue-600 transition");
                                    });
                                });
                            });
                        });
                    });
                }))
            })
            .get("/:todo_id", |State(db): State<DB>, _: AuthenticatedUser, Path(todo_id): Path<u64>| -> Result<HTMLElement, ErrorResponse> {
                let todo = match crate::db::get_todo(&db, todo_id)? {
                    Some(x) => x,
                    None => {
                        return Err(ErrorStatusCode::NotFound.into())
                    }
                };
    
                Ok(html::html(|| {
                    Title(format!("TodoApp | Todo #{}", todo.id), ());
    
                    html::body(|| {
                        html::div(|| {
                            html::class("min-h-screen bg-gray-100 p-6 flex items-center justify-center");
    
                            html::div(|| {
                                html::class("bg-white p-8 rounded shadow-lg w-full max-w-md");
    
                                html::h1(|| {
                                    html::content(&todo.title);
                                    html::class("text-3xl font-bold text-center text-blue-600 mb-4");
                                });
    
                                if let Some(desc) = &todo.description {
                                    html::p(|| {
                                        html::content(desc);
                                        html::class("text-gray-600 text-center mb-4");
                                    });
                                }
    
                                html::p(|| {
                                    html::content(format!("Completed: {}", todo.is_done));
                                    html::class("text-gray-600 text-center mb-4");
                                });
                            });
                        });
                    });
                }))
            })
    }
    

    pub fn home_routes() -> Scope {
        Scope::new()
            .get("/login", |auth: Option<AuthenticatedUser>| -> Result<HTMLElement, Redirect> {
                if auth.is_some() {
                    return Err(Redirect::see_other("/todos"));
                }
    
                Ok(html::html(|| {
                    Title("TodoApp | Login", ());
    
                    html::body(|| {
                        html::div(|| {
                            html::class("min-h-screen flex items-center justify-center bg-gray-100");
    
                            html::div(|| {
                                html::class("bg-white p-8 rounded shadow-lg w-full max-w-md");
    
                                html::h1(|| {
                                    html::content("Login to TodoApp");
                                    html::class("text-3xl font-extrabold text-center text-blue-600 mb-6");
                                });
    
                                html::form(|| {
                                    html::attr("method", "post");
                                    html::attr("action", "/api/login");
    
                                    html::input(|| {
                                        html::attr("type", "text");
                                        html::attr("placeholder", "Username");
                                        html::attr("name", "username");
                                        html::class("mt-4 p-3 border border-gray-300 rounded w-full focus:ring-2 focus:ring-blue-500 transition-all duration-300 ease-in-out");
                                    });
    
                                    html::button(|| {
                                        html::content("Login");
                                        html::class("mt-6 p-3 bg-green-500 text-white font-semibold rounded w-full hover:bg-green-600 transition duration-300 ease-in-out transform hover:scale-105");
                                    });
                                });
                            });
                        });
                    });
                }))
            })
            .get("/me", |AuthenticatedUser(user): AuthenticatedUser| {
                html::html(|| {
                    Title("TodoApp | Me", ());
    
                    html::body(|| {
                        html::div(|| {
                            html::class("min-h-screen flex items-center justify-center bg-gray-100");
    
                            html::div(|| {
                                html::class("bg-white p-8 rounded shadow-lg w-full max-w-md");
    
                                html::h1(|| {
                                    html::content(format!("Welcome, {}!", user.username));
                                    html::class("text-3xl font-extrabold text-center text-blue-600 mb-6");
                                });
    
                                html::p(|| {
                                    html::content(format!("User ID: {}", user.id));
                                    html::class("mt-4 text-gray-600 text-center text-lg");
                                });
    
                                html::p(|| {
                                    html::content("Have a great day using TodoApp!");
                                    html::class("mt-4 text-gray-500 text-center text-sm");
                                });
                            });
                        });
                    });
                })
            })
    }
    
}

#[allow(non_snake_case)]
mod components {
    use http1_web::html::{self, element::HTMLElement, IntoChildren};

    pub fn Title(title: impl Into<String>, content: impl IntoChildren) -> HTMLElement {
        html::head(|| {
            html::title(title.into());
            html::meta(|| html::attr("charset", "UTF-8"));
            html::script(|| {
                html::attr("src", "https://cdn.tailwindcss.com");
            });

            content.into_children();
        })
    }
}

mod db {
    use std::{
        any::Any,
        collections::HashMap,
        fmt::Display,
        sync::{atomic::AtomicU64, Arc, RwLock},
    };

    use http1_web::impl_serde_struct;

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum Table {
        User,
        Todo,
        Session,
    }

    struct Id(AtomicU64);
    impl Id {
        const fn new() -> Self {
            Id(AtomicU64::new(0))
        }

        fn next(&self) -> u64 {
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
        }
    }

    #[derive(Debug, Clone)]
    pub struct User {
        pub id: u64,
        pub username: String,
    }

    impl_serde_struct!(User => {
        id: u64,
        username: String,
   });

    #[derive(Debug, Clone)]
    pub struct Todo {
        pub id: u64,
        pub title: String,
        pub description: Option<String>,
        pub is_done: bool,
        pub user_id: u64,
    }

    impl_serde_struct!(Todo => {
        id: u64,
        title: String,
        description: Option<String>,
        is_done: bool,
        user_id: u64,
   });

    #[derive(Debug, Clone)]
    pub struct Session {
        pub id: String,
        pub user_id: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum Key {
        Number(u64),
        String(String),
    }

    #[derive(Clone)]
    pub struct DB(Arc<RwLock<HashMap<Table, HashMap<Key, Box<dyn Any + Send + Sync>>>>>);

    impl DB {
        pub fn new() -> Self {
            let tables = HashMap::from_iter([
                (Table::User, Default::default()),
                (Table::Todo, Default::default()),
                (Table::Session, Default::default()),
            ]);

            DB(Arc::new(RwLock::new(tables)))
        }
    }

    static NEXT_USER_ID: Id = Id::new();
    static NEXT_TODO_ID: Id = Id::new();

    #[derive(Debug)]
    pub enum DBError {
        FailedToRead,
        FailedToWrite,
    }

    impl std::error::Error for DBError {}

    impl Display for DBError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                DBError::FailedToRead => write!(f, "failed to open database to read"),
                DBError::FailedToWrite => write!(f, "failed to open database to write"),
            }
        }
    }

    pub fn insert_user(db: &DB, username: String) -> Result<User, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        let id = NEXT_USER_ID.next();
        let user = User { id, username };
        records.insert(Key::Number(id), Box::new(user.clone()));
        Ok(user)
    }

    pub fn update_user(db: &DB, user: User) -> Result<Option<User>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        match records
            .get_mut(&Key::Number(user.id))
            .and_then(|x| x.downcast_mut::<User>())
        {
            Some(user_to_update) => {
                user_to_update.username = user.username;
                Ok(Some(user_to_update.clone()))
            }
            None => Ok(None),
        }
    }

    pub fn delete_user(db: &DB, id: u64) -> Result<Option<User>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        let deleted = records
            .remove(&Key::Number(id))
            .and_then(|x| x.downcast::<User>().ok())
            .map(|x| *x);

        Ok(deleted)
    }

    pub fn get_user(db: &DB, id: u64) -> Result<Option<User>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let records = lock.get(&Table::User).expect("user table should exists");

        records
            .get(&Key::Number(id))
            .and_then(|x| x.downcast_ref::<User>())
            .cloned()
            .map(Ok)
            .transpose()
    }

    pub fn get_all_user(db: &DB) -> Result<Vec<User>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let records = lock.get(&Table::User).expect("user table should exists");

        let users = records
            .values()
            .filter_map(|x| x.downcast_ref::<User>())
            .cloned()
            .collect::<Vec<_>>();

        Ok(users)
    }

    pub fn insert_todo(
        db: &DB,
        title: String,
        description: Option<String>,
        user_id: u64,
    ) -> Result<Todo, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Todo)
            .expect("todos table should exists");

        let id = NEXT_TODO_ID.next();
        let todo: Todo = Todo {
            id,
            title,
            description,
            is_done: false,
            user_id,
        };

        records.insert(Key::Number(id), Box::new(todo.clone()));
        Ok(todo)
    }

    pub fn update_todo(db: &DB, todo: Todo) -> Result<Option<Todo>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Todo)
            .expect("todo table should exists");

        match records
            .get_mut(&Key::Number(todo.id))
            .and_then(|x| x.downcast_mut::<Todo>())
        {
            Some(to_update) => {
                *to_update = todo;
                Ok(Some(to_update.clone()))
            }
            None => Ok(None),
        }
    }

    pub fn delete_todo(db: &DB, id: u64) -> Result<Option<Todo>, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Todo)
            .expect("todos table should exists");

        let deleted = records
            .remove(&Key::Number(id))
            .and_then(|x| x.downcast::<Todo>().ok())
            .map(|x| *x);

        Ok(deleted)
    }

    pub fn get_todo(db: &DB, id: u64) -> Result<Option<Todo>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let todos = lock.get(&Table::Todo).expect("todos table should exists");

        todos
            .get(&Key::Number(id))
            .and_then(|x| x.downcast_ref::<Todo>())
            .cloned()
            .map(Ok)
            .transpose()
    }

    pub fn get_all_todos(db: &DB, user_id: u64) -> Result<Vec<Todo>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let todos = lock.get(&Table::Todo).expect("todos table should exists");

        let todos = todos
            .values()
            .filter_map(|x| x.downcast_ref::<Todo>())
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect::<Vec<_>>();

        Ok(todos)
    }

    pub fn create_session(db: &DB, user_id: u64) -> Result<Session, DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Session)
            .expect("sessions table should exists");



        let id = http1::rng::sequence::<http1::rng::random::Alphanumeric>()
            .take(32)
            .collect::<String>();


            log::warn!("Creating session: {id}");

        let session = Session {
            id: id.clone(),
            user_id,
        };
        records.insert(Key::String(id), Box::new(session.clone()));
        Ok(session)
    }

    pub fn get_session_by_id(db: &DB, session_id: String) -> Result<Option<Session>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let sessions = lock
            .get(&Table::Session)
            .expect("sessions table should exists");

        sessions
            .get(&Key::String(session_id))
            .and_then(|x| x.downcast_ref::<Session>())
            .cloned()
            .map(Ok)
            .transpose()
    }

    pub fn get_session_user(db: &DB, session_id: String) -> Result<Option<User>, DBError> {
        log::warn!("Search session: {session_id}");

        let session = match get_session_by_id(db, session_id)? {
            Some(s) => s,
            None => return Ok(None),
        };

        get_user(db, session.user_id)
    }

    pub fn remove_session(db: &DB, session_id: String) -> Result<(), DBError> {
        let mut lock = db.0.write().map_err(|_| DBError::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Session)
            .expect("sessions table should exists");

        records.remove(&Key::String(session_id));
        Ok(())
    }
}
