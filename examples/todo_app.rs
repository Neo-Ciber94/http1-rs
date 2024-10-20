use db::DB;
use http1::server::Server;
use http1_web::{app::App, middleware::logging::Logging};
use routes::{api_routes, auth_routes, home_routes};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let addr = "127.0.0.1:5000".parse().unwrap();

    let app = App::new()
        .state(DB::new())
        .middleware(Logging)
        .scope("/api", api_routes())
        .scope("/", home_routes())
        .scope("/auth", auth_routes());

    Server::new(addr)
        .on_ready(|addr| log::info!("Listening on http://{addr}"))
        .start(app)
}

const COOKIE_SESSION_NAME: &str = "auth-session-id";

mod routes {
    use http1::{body::Body, response::Response};
    use http1_web::{
        app::Scope,
        cookies::{Cookie, Cookies},
        error_response::{ErrorResponse, ErrorStatusCode},
        forms::form::Form,
        from_request::FromRequestRef,
        html::{self, element::HTMLElement},
        impl_serde_struct,
        into_response::IntoResponse,
        redirect::Redirect,
        state::State,
    };

    use crate::{
        components::Title,
        db::{User, DB},
        COOKIE_SESSION_NAME,
    };

    #[derive(Debug)]
    struct AuthenticatedUser(pub User);

    impl FromRequestRef for AuthenticatedUser {
        type Rejection = ErrorStatusCode;

        fn from_request_ref(
            req: &http1::request::Request<http1::body::Body>,
        ) -> Result<Self, Self::Rejection> {
            let State(db) = State::<DB>::from_request_ref(req)
                .map_err(|_| ErrorStatusCode::InternalServerError)?;
            let cookies = Cookies::from_request_ref(req).unwrap_or_default();
            let session_cookie = cookies
                .get(super::COOKIE_SESSION_NAME)
                .ok_or_else(|| ErrorStatusCode::Unauthorized)?;
            match crate::db::get_session_user(&db, session_cookie.value().to_owned()) {
                Ok(Some(user)) => Ok(AuthenticatedUser(user)),
                _ => Err(ErrorStatusCode::Unauthorized),
            }
        }
    }

    pub fn home_routes() -> Scope {
        Scope::new().get("/", || "Hello World!")
    }

    pub fn api_routes() -> Scope {
        #[derive(Debug)]
        struct LoginUser {
            pub username: String,
        }

        impl_serde_struct!(LoginUser => {
             username: String
        });

        Scope::new().post(
            "/login",
            |State(db): State<DB>,
             input: Form<LoginUser>|
             -> Result<Response<Body>, ErrorResponse> {
                let user = crate::db::insert_user(&db, input.into_inner().username)?;
                let session = crate::db::create_session(&db, user.id)?;

                log::info!("User created: {user:?}");

                Ok((
                    Redirect::temporary_redirect("/me"),
                    Cookie::new(COOKIE_SESSION_NAME, session.id.clone()).build(),
                )
                    .into_response())
            },
        )
    }

    pub fn auth_routes() -> Scope {
        Scope::new()
            .get("/login", |auth: Option<AuthenticatedUser>| -> Result<HTMLElement, Redirect> {

                if auth.is_some() {
                    return Err(Redirect::see_other("/"));
                }

                Ok(html::html(|| {
                    Title("TodoApp | Login", ());
    
                    html::body(|| {
                        html::div(|| {
                            html::h1(|| {
                                html::content("Login to TodoApp");
                                html::class("text-2xl font-bold text-center text-blue-600");
                            });
    
                            html::form(|| {
                                html::attr("method", "post");
                                html::attr("action", "/api/login");

                                html::input(|| {
                                    html::attr("type", "text");
                                    html::attr("placeholder", "Username");
                                    html::attr("name", "username");
                                    html::class("mt-4 p-2 border border-gray-300 rounded w-full");
                                });
    
                                html::button(|| {
                                    html::content("Login");
                                    html::class("mt-4 p-2 bg-green-500 text-white rounded w-full hover:bg-green-600");
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
                            html::h1(|| {
                                html::content(format!("Welcome, {}!", user.username));
                                html::class("text-2xl font-bold text-center text-blue-600");
                            });
    
                            html::p(|| {
                                html::content(format!("User ID: {}", user.id));
                                html::class("mt-4 text-gray-600 text-center");
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

    #[derive(Debug, Clone)]
    pub struct Todo {
        pub id: u64,
        pub title: String,
        pub description: Option<String>,
        pub is_done: bool,
        pub user_id: u64,
    }

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

    pub fn get_all_todos(db: &DB) -> Result<Vec<Todo>, DBError> {
        let lock = db.0.read().map_err(|_| DBError::FailedToRead)?;
        let todos = lock.get(&Table::Todo).expect("todos table should exists");

        let todos = todos
            .values()
            .filter_map(|x| x.downcast_ref::<Todo>())
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
