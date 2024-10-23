use db::DB;
use http1::server::Server;
use http1_web::{
    app::App,
    middleware::{logging::Logging, redirection::Redirection},
};
use routes::{api_routes, home_routes, not_found, todos_routes};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let addr = "localhost:5000";

    let app = App::new()
        .state(DB::new())
        .middleware(Logging)
        .middleware(Redirection::new("/", "/login"))
        .scope("/api", api_routes())
        .scope("/", home_routes())
        .scope("/todos", todos_routes())
        .fallback(not_found);

    Server::new(addr)
        .on_ready(|addr| log::info!("Listening on http://{addr}"))
        .start(app)
}

const COOKIE_SESSION_NAME: &str = "auth_session";

mod routes {
    use std::time::Duration;

    use datetime::DateTime;
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
        path::Path,
        redirect::Redirect,
        state::State,
    };

    use crate::{
        components::{Head, Layout, Title},
        db::{User, DB},
        COOKIE_SESSION_NAME,
    };

    #[derive(Debug)]
    pub struct AuthenticatedUser(pub User);

    pub enum AuthenticatedUserRejection {
        StateError,
        RedirectToLogin,
    }

    impl IntoResponse for AuthenticatedUserRejection {
        fn into_response(self) -> Response<Body> {
            match self {
                AuthenticatedUserRejection::StateError => {
                    ErrorStatusCode::InternalServerError.into_response()
                }
                AuthenticatedUserRejection::RedirectToLogin => {
                    Redirect::see_other("/login").into_response()
                }
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
                }
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
                            .expires(DateTime::now_utc() + Duration::from_secs(60 * 60))
                            .build(),
                    )
                        .into_response())
                },
            )
            .get(
                "/logout",
                |State(db): State<DB>, mut cookies: Cookies, auth: Option<AuthenticatedUser>|
                 -> Result<Response<Body>, ErrorResponse> {
                   
                   if auth.is_none() {
                    return Ok(Redirect::see_other("/todos").into_response());
                   }

                   let session_cookie = match cookies.get(COOKIE_SESSION_NAME) {
                    Some(c) => c,
                    None => {
                        return Ok(Redirect::see_other("/todos").into_response());
                    }
                   };

                   crate::db::remove_session(&db, session_cookie.value().to_owned())?;
                   cookies.del(COOKIE_SESSION_NAME);
                   
                   Ok((Redirect::see_other("/"), cookies).into_response())
                },
            )
            .post(
                "/todos/create",
                |State(db): State<DB>,
                 AuthenticatedUser(user): AuthenticatedUser,
                 Form(todo): Form<CreateTodo>|
                 -> Result<Redirect, ErrorResponse> {
                    crate::db::insert_todo(
                        &db,
                        todo.title,
                        todo.description.filter(|x| !x.is_empty()),
                        user.id,
                    )?;

                    Ok(Redirect::see_other("/todos"))
                },
            )
            .post(
                "/todos/update",
                |State(db): State<DB>,
                 Form(form): Form<UpdateTodo>|
                 -> Result<Redirect, ErrorResponse> {
                    let mut todo = match crate::db::get_todo(&db, form.id)? {
                        Some(x) => x,
                        None => return Err(ErrorStatusCode::NotFound.into()),
                    };

                    // Update fields
                    todo.title = form.title;
                    todo.description = form.description;

                    // Save
                    crate::db::update_todo(&db, todo)?;
                    Ok(Redirect::see_other("/todos"))
                },
            )
            .post(
                "/todos/delete/:todo_id",
                |State(db): State<DB>,
                 Path(todo_id): Path<u64>|
                 -> Result<Redirect, ErrorResponse> {
                    crate::db::delete_todo(&db, todo_id)?;

                    Ok(Redirect::see_other("/todos"))
                },
            )
            .post(
                "/todos/toggle/:todo_id",
                |State(db): State<DB>,
                 Path(todo_id): Path<u64>|
                 -> Result<Redirect, ErrorResponse> {
                    let mut todo = match crate::db::get_todo(&db, todo_id)? {
                        Some(x) => x,
                        None => return Err(ErrorStatusCode::NotFound.into()),
                    };

                    todo.is_done = !todo.is_done;
                    crate::db::update_todo(&db, todo)?;

                    Ok(Redirect::see_other("/todos"))
                },
            )
    }

    pub fn todos_routes() -> Scope {
        Scope::new()
            .get("/", |State(db): State<DB>, auth: AuthenticatedUser| -> Result<HTMLElement, ErrorResponse> {
                let AuthenticatedUser(user) = &auth;
                let todos = crate::db::get_all_todos(&db, user.id)?;

                Ok(html::html(|| {
                    Head(|| {
                        Title("TodoApp | Todos");
                        html::style(r#"
                         .animate-fade-in {
                            opacity: 0; 
                            animation: fadeIn 300ms ease-in forwards;
                        }

                         @keyframes fadeIn {
                                to {
                                    opacity: 1;
                                }
                            }
                        "#);
                    });
                
                    html::body(|| {
                        Layout(Some(auth), || {
                            html::div(|| {
                                html::class("min-h-screen bg-gray-100 p-6");
                    
                                // Main heading
                                html::h1(|| {
                                    html::content("Your Todos");
                                    html::class("text-3xl font-bold text-center text-blue-600 mb-8");
                                });
                    
                                // Link to create a new todo
                                html::a(|| {
                                    html::attr("href", "/todos/create");
                                    html::content("Create New Todo");
                                    html::class("mt-4 p-3 bg-green-500 text-white rounded hover:bg-green-600 transition inline-block shadow-md mb-5");
                                });
                    
                                // Todo items container
                                html::div(|| {
                                    todos.iter().for_each(|todo| {
                                        html::div(|| {
                                            html::class("bg-white shadow-lg rounded-lg p-4 mb-4 border border-gray-200 flex flex-row justify-between animate-fade-in");
                    
                                            html::div(|| {
                                                let is_done_class =  if todo.is_done { 
                                                    "line-through opacity-50 italic" 
                                                } 
                                                else  { 
                                                    "" 
                                                };
    
                                                // Todo title
                                                html::h2(|| {
                                                    html::content(&todo.title);
                                                    html::class(format!("text-xl font-bold text-gray-800 flex-grow {is_done_class}"));
                                                });
                    
                        
                                                // Todo description
                                                if let Some(desc) = &todo.description {
                                                    html::p(|| {
                                                        html::content(desc);
                                                        html::class(format!("text-gray-600 mt-2 {is_done_class}"));
                                                    });
                                                }
                                                else {
                                                    html::p(|| {
                                                        html::content("<no description>");
                                                        html::class("text-gray-600 opacity-50 italic mt-2");
                                                    });
                                                }
                                            });
    
                                            html::div(|| {
                                                html::class("flex flex-col gap-2");
                                                html::styles("min-width: 100px");
    
                                                html::form(|| {
                                                    html::attr("method", "post");
                                                    html::attr("action", format!("/api/todos/toggle/{}", todo.id));
                                                    html::class("flex items-center m-0");
                        
                                                    html::button(|| {
                                                        if todo.is_done {
                                                            html::content("Completed");
                                                            html::class("p-2 bg-red-500 text-white rounded hover:bg-green-600 transition w-full"); 
                                                        } else {
                                                            html::content("Pending");
                                                            html::class("p-2 bg-yellow-500 text-white rounded hover:bg-yellow-600 transition w-full");
                                                        }
                                                    });
                                                });
    
                                                // Edit button
                                                html::a(|| {
                                                    html::attr("href", format!("/todos/edit/{}", todo.id));
                                                    html::content("Edit");
                                                    html::class("p-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition flex justify-center");
                                                });
                        
                                                // Delete form
                                                html::form(|| {
                                                    html::attr("method", "post");
                                                    html::attr("action", format!("/api/todos/delete/{}", todo.id));
                                                    html::class("flex justify-center");
                        
                                                    html::button(|| {
                                                        html::content("Delete");
                                                        html::class("p-2 bg-red-500 text-white rounded hover:bg-red-600 transition w-full");
                                                    });
                                                });
                                            });
                                        });
                                    });
                                });
                            });
                        });
                    });
                })
                )
            })
            .get("/create", | auth: AuthenticatedUser| {
                html::html(|| {
                    Head(|| {
                        Title("TodoApp | Create Todo");
                    });
    
                    html::body(|| {
                        Layout(Some(auth), || {
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
                                            html::attr("required", true);
                                            html::class("mt-4 p-3 border border-gray-300 rounded w-full");
                                            html::attr("minlength", 3);
                                            html::attr("pattern", ".*\\S.*");
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
                    });
                })
            })
            .get("/edit/:todo_id", |State(db): State<DB>, auth: AuthenticatedUser, Path(todo_id): Path<u64>| -> Result<HTMLElement, ErrorResponse> {
                let todo = match crate::db::get_todo(&db, todo_id)? {
                    Some(x) => x,
                        None => {
                        return Err(ErrorStatusCode::NotFound.into())
                    }
                };
    
                Ok(html::html(|| {
                    Head(|| {
                        Title("TodoApp | Edit Todo");
                    });
    
                    html::body(|| {
                        Layout(Some(auth), || {
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
                                            html::attr("required", true);
                                            html::class("mt-4 p-3 border border-gray-300 rounded w-full");
                                        });
        
                                        html::textarea(|| {
                                            html::attr("placeholder", "Description (optional)");
                                            html::attr("name", "description");
                                            html::attr("value", todo.description.clone().unwrap_or_default());
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
                    });
                }))
            })
            .get("/:todo_id", |State(db): State<DB>, auth: AuthenticatedUser, Path(todo_id): Path<u64>| -> Result<HTMLElement, ErrorResponse> {
                let todo = match crate::db::get_todo(&db, todo_id)? {
                    Some(x) => x,
                    None => {
                        return Err(ErrorStatusCode::NotFound.into())
                    }
                };
    
                Ok(html::html(|| {
                    Head(|| {
                        Title(format!("TodoApp | {}", todo.title));
                    });
    
                    html::body(|| {
                        Layout(Some(auth), || {
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
                    Head(|| {
                        Title("TodoApp | Login");
                    });

                    html::body(|| {
                        Layout(auth, || {
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
                                            html::attr("required", true);
                                            html::attr("minlength", 3);
                                            html::attr("pattern", ".*\\S.*");
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
                    });
                }))
            })
    }

    pub fn not_found() -> HTMLElement {
        html::html(|| {
            Head(|| {
                Title("Todo App | Not Found");
            });

            html::body(|| {
                html::div(|| {
                    html::class("min-h-screen bg-gray-100 flex items-center justify-center");

                    html::div(|| {
                        html::class("bg-white p-8 rounded shadow-lg text-center");

                        // Heading
                        html::h1(|| {
                            html::content("404 Not Found");
                            html::class("text-4xl font-bold text-red-600");
                        });

                        // Description
                        html::p(|| {
                            html::content("The page you are looking for does not exist.");
                            html::class("text-gray-600 mt-4");
                        });

                        // Back to home link
                        html::a(|| {
                            html::attr("href", "/");
                            html::content("Go to Home");
                            html::class("mt-6 inline-block p-3 bg-blue-500 text-white rounded hover:bg-blue-600 transition");
                        });
                    });
                });
            });
        })
    }
}

#[allow(non_snake_case)]
mod components {
    use http1_web::html::{self, element::HTMLElement, IntoChildren};

    use crate::routes::AuthenticatedUser;

    pub fn Head(content: impl IntoChildren) -> HTMLElement {
        html::head(|| {
            content.into_children();

            html::meta(|| html::attr("charset", "UTF-8"));
            html::script(|| {
                html::attr("src", "https://cdn.tailwindcss.com");
            });
        })
    }

    pub fn Title(title: impl Into<String>) -> HTMLElement {
        html::title(title.into())
    }

    pub fn Layout(auth: Option<AuthenticatedUser>, content: impl IntoChildren) -> HTMLElement {
        html::div(|| {
            html::header(|| {
                html::class("w-full h-16 shadow fixed top-0 left-0 flex flex-row p-2 justify-end items-center");
                
                html::h4(|| {
                    html::content("TodoApp");
                    html::class("text-xl font-bold text-blue-500");
                });

                if let Some(AuthenticatedUser(user)) = auth {
                html::div(|| {
                    html::class("flex flex-row gap-2 items-center");
                        html::h4(|| {
                            html::content(user.username);
                            html::class("font-bold text-xl");
                        });
                        html::a(|| {
                            html::attr("href", "/api/logout");

                            html::button(|| {
                                html::content("Logout");
                                html::class("p-3 bg-black text-white rounded hover:bg-slate-800 transition inline-block shadow-md");
                            });
                        });
                    });
                }
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
    pub struct ValidationError {
        field: String,
        message: String,
    }

    impl ValidationError {
        pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
            ValidationError {
                field: field.into(),
                message: message.into(),
            }
        }

        pub fn field(&self) -> &str {
            &self.field
        }

        pub fn message(&self) -> &str {
            &self.message
        }
    }

    #[derive(Debug)]
    pub enum Error {
        FailedToRead,
        FailedToWrite,
        Validation(ValidationError),
    }

    impl std::error::Error for Error {}

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::FailedToRead => write!(f, "failed to open database to read"),
                Error::FailedToWrite => write!(f, "failed to open database to write"),
                Error::Validation(ValidationError { field, message }) => {
                    write!(f, "invalid `{field}`: {message}")
                }
            }
        }
    }

    struct Validation;
    impl Validation {
        pub fn validate_user(user: &mut User) -> Result<(), Error> {
            if user.username.trim().is_empty() {
                return Err(Error::Validation(ValidationError::new(
                    "username",
                    "cannot be empty",
                )));
            }

            user.username = user.username.trim().into();

            Ok(())
        }

        pub fn validate_todo(todo: &mut Todo) -> Result<(), Error> {
            if todo.title.trim().is_empty() {
                return Err(Error::Validation(ValidationError::new(
                    "title",
                    "cannot be empty",
                )));
            }

            if let Some(description) = todo.description.as_deref().filter(|x| x.is_empty()) {
                if description.trim().is_empty() {
                    return Err(Error::Validation(ValidationError::new(
                        "description",
                        "cannot be empty",
                    )));
                }
            }

            todo.title = todo.title.trim().into();
            todo.description = todo.description.as_deref().map(|x| x.trim().into());

            Ok(())
        }
    }

    pub fn insert_user(db: &DB, username: String) -> Result<User, Error> {
        let mut lock = db.0.write().map_err(|_| Error::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        let id = NEXT_USER_ID.next();
        let mut user = User {
            id,
            username: username.trim().into(),
        };

        Validation::validate_user(&mut user)?;

        records.insert(Key::Number(id), Box::new(user.clone()));
        Ok(user)
    }

    pub fn update_user(db: &DB, mut user: User) -> Result<Option<User>, Error> {
        let mut lock = db.0.write().map_err(|_| Error::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        Validation::validate_user(&mut user)?;

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

    pub fn delete_user(db: &DB, id: u64) -> Result<Option<User>, Error> {
        let mut lock = db.0.write().map_err(|_| Error::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::User)
            .expect("user table should exists");

        let deleted = records
            .remove(&Key::Number(id))
            .and_then(|x| x.downcast::<User>().ok())
            .map(|x| *x);

        Ok(deleted)
    }

    pub fn get_user(db: &DB, id: u64) -> Result<Option<User>, Error> {
        let lock = db.0.read().map_err(|_| Error::FailedToRead)?;
        let records = lock.get(&Table::User).expect("user table should exists");

        records
            .get(&Key::Number(id))
            .and_then(|x| x.downcast_ref::<User>())
            .cloned()
            .map(Ok)
            .transpose()
    }

    pub fn get_all_user(db: &DB) -> Result<Vec<User>, Error> {
        let lock = db.0.read().map_err(|_| Error::FailedToRead)?;
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
    ) -> Result<Todo, Error> {
        let mut lock = db.0.write().map_err(|_| Error::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Todo)
            .expect("todos table should exists");

        let id = NEXT_TODO_ID.next();
        let mut todo = Todo {
            id,
            title,
            description,
            is_done: false,
            user_id,
        };

        Validation::validate_todo(&mut todo)?;
        records.insert(Key::Number(id), Box::new(todo.clone()));
        Ok(todo)
    }

    pub fn update_todo(db: &DB, mut todo: Todo) -> Result<Option<Todo>, Error> {
        let mut lock = db.0.write().map_err(|_| Error::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Todo)
            .expect("todo table should exists");

        Validation::validate_todo(&mut todo)?;

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

    pub fn delete_todo(db: &DB, id: u64) -> Result<Option<Todo>, Error> {
        let mut lock = db.0.write().map_err(|_| Error::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Todo)
            .expect("todos table should exists");

        let deleted = records
            .remove(&Key::Number(id))
            .and_then(|x| x.downcast::<Todo>().ok())
            .map(|x| *x);

        Ok(deleted)
    }

    pub fn get_todo(db: &DB, id: u64) -> Result<Option<Todo>, Error> {
        let lock = db.0.read().map_err(|_| Error::FailedToRead)?;
        let todos = lock.get(&Table::Todo).expect("todos table should exists");

        todos
            .get(&Key::Number(id))
            .and_then(|x| x.downcast_ref::<Todo>())
            .cloned()
            .map(Ok)
            .transpose()
    }

    pub fn get_all_todos(db: &DB, user_id: u64) -> Result<Vec<Todo>, Error> {
        let lock = db.0.read().map_err(|_| Error::FailedToRead)?;
        let todos = lock.get(&Table::Todo).expect("todos table should exists");

        let todos = todos
            .values()
            .filter_map(|x| x.downcast_ref::<Todo>())
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect::<Vec<_>>();

        Ok(todos)
    }

    pub fn create_session(db: &DB, user_id: u64) -> Result<Session, Error> {
        let mut lock = db.0.write().map_err(|_| Error::FailedToWrite)?;
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

    pub fn get_session_by_id(db: &DB, session_id: String) -> Result<Option<Session>, Error> {
        let lock = db.0.read().map_err(|_| Error::FailedToRead)?;
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

    pub fn get_session_user(db: &DB, session_id: String) -> Result<Option<User>, Error> {
        log::warn!("Search session: {session_id}");

        let session = match get_session_by_id(db, session_id)? {
            Some(s) => s,
            None => return Ok(None),
        };

        get_user(db, session.user_id)
    }

    pub fn remove_session(db: &DB, session_id: String) -> Result<(), Error> {
        let mut lock = db.0.write().map_err(|_| Error::FailedToWrite)?;
        let records = lock
            .get_mut(&Table::Session)
            .expect("sessions table should exists");

        records.remove(&Key::String(session_id));
        Ok(())
    }
}

mod kv {
    use std::{fmt::Display,  path::{Path, PathBuf}};
    use http1_web::serde::{de::Deserialize, json::value::JsonValue, ser::Serialize};

    #[derive(Debug)]
    pub struct SetValueError;

    impl Display for SetValueError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "failed to set value")
        }
    }

    #[derive(Debug)]
    pub struct KeyValueDatabase(PathBuf); 

    impl KeyValueDatabase {
        pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
            let cwd = std::env::current_dir()?;
            let file_path = cwd.join(path);

            assert!(file_path.is_file(), "is not a file: {file_path:?}");

            if (!file_path.exists()) {
                let mut ancestors  = file_path.ancestors();
                ancestors.next();

                if let Some(dir) = ancestors.next() {
                    std::fs::create_dir_all(dir)?;
                }
            }

            Ok(KeyValueDatabase(file_path))
        }

        fn tap<F, R>(&self, f: F) -> std::io::Result<R> where F: FnOnce(&mut JsonValue) -> std::io::Result<R> {
            let bytes = std::fs::read(self.0.as_path())?;
            let mut json = if bytes.is_empty() {
                JsonValue::Object(Default::default())
            } else {
                http1_web::serde::json::from_bytes::<JsonValue>(bytes).map_err(|err| std::io::Error::other(err))?
            };

            let result = f(&mut json);
            std::fs::write(self.0.as_path(), json.to_string())?;
            result
        }

        pub fn set<T: Serialize>(&self, key: impl AsRef<str>, value: T)  -> std::io::Result<()> {
            self.tap(|json| {
                json[key.as_ref()] = http1_web::serde::json::to_value(&value).map_err(|err| std::io::Error::other(err))?;
                Ok(())
            })
        }

        pub fn get<T: Deserialize>(&self, key: impl AsRef<str>) -> std::io::Result<Option<T>> {
            self.tap(|json | {
                let value = match json.get(key.as_ref()) {
                    Some(x) => x,
                    None => return Ok(None),
                };

                http1_web::serde::json::from_value(value.clone()).map_err(|err| std::io::Error::other(err))
            })
        }

        pub fn del(&self, key: impl AsRef<str>) -> std::io::Result<()> {
            self.tap(|json| {
                json.remove(key.as_ref());
                Ok(())
            })
        }
    }
}