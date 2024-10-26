
use http1::server::Server;
use http1_web::{
    app::App, fs::ServeDir, middleware::{logging::Logging, redirection::Redirection}
};
use kv::KeyValueDatabase;
use routes::{api_routes,  page_routes};

fn main() -> std::io::Result<()> {
    log::set_logger(log::ConsoleLogger);

    let addr = "localhost:5000";

    let app = App::new()
        .state(KeyValueDatabase::new("examples/todo_app/db.json").unwrap())
        .middleware(Logging)
        .middleware(Redirection::new("/", "/login"))
        .get("/*", ServeDir::new("/", "examples/todo_app/public"))
        .scope("/api", api_routes())
        .get("/hello", || "Adios")
        .scope("/", page_routes());

        dbg!(&app);
        
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
        into_response::{IntoResponse, NotFound},
        path::Path,
        redirect::Redirect,
        state::State,
    };

    use crate::{
        components::{Head, Layout, Title}, models::User, kv::KeyValueDatabase, COOKIE_SESSION_NAME
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
            let State(db) = State::<KeyValueDatabase>::from_request_ref(req)
                .inspect_err(|err| {
                    log::error!("Failed to get database: {err}");
                })
                .map_err(|_| AuthenticatedUserRejection::StateError)?;

            let cookies = Cookies::from_request_ref(req).unwrap_or_default();

            let session_cookie = cookies
                .get(super::COOKIE_SESSION_NAME)
                .ok_or_else(|| AuthenticatedUserRejection::RedirectToLogin)?;

            let session_id = session_cookie.value();

            match crate::models::get_session_user(&db, session_id.to_owned()) {
                Ok(Some(user)) => Ok(AuthenticatedUser(user)),
                x => {
                    log::error!("user session `{session_id}` not found: {x:?}");
                    Err(AuthenticatedUserRejection::RedirectToLogin)
                }
            }
        }
    }

    pub fn page_routes() -> Scope {
        Scope::new()
            .scope("/", home_routes())
            .scope("/todos", todos_routes())
            .fallback(not_found)
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
                |State(db): State<KeyValueDatabase>,
                 Form(input): Form<LoginUser>|
                 -> Result<Response<Body>, ErrorResponse> {
                    let user = match crate::models::get_user_by_username(&db, &input.username)? {
                        Some(x) => x,
                        None => {
                            let new_user = crate::models::insert_user(&db, input.username)?;        
                            log::info!("User created: {new_user:?}");
                            new_user
                        },
                    };

                    let session = crate::models::create_session(&db, user.id)?;

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
                |State(db): State<KeyValueDatabase>, mut cookies: Cookies, auth: Option<AuthenticatedUser>|
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

                   crate::models::remove_session(&db, session_cookie.value().to_owned())?;
                   cookies.del(COOKIE_SESSION_NAME);
                   
                   Ok((Redirect::see_other("/"), cookies).into_response())
                },
            )
            .post(
                "/todos/create",
                |State(db): State<KeyValueDatabase>,
                 AuthenticatedUser(user): AuthenticatedUser,
                 Form(todo): Form<CreateTodo>|
                 -> Result<Redirect, ErrorResponse> {
                    crate::models::insert_todo(
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
                |State(db): State<KeyValueDatabase>,
                 Form(form): Form<UpdateTodo>|
                 -> Result<Redirect, ErrorResponse> {
                    let mut todo = match crate::models::get_todo(&db, form.id)? {
                        Some(x) => x,
                        None => return Err(ErrorStatusCode::NotFound.into()),
                    };

                    // Update fields
                    todo.title = form.title;
                    todo.description = form.description;

                    // Save
                    crate::models::update_todo(&db, todo)?;
                    Ok(Redirect::see_other("/todos"))
                },
            )
            .post(
                "/todos/delete/:todo_id",
                |State(db): State<KeyValueDatabase>,
                 Path(todo_id): Path<u64>|
                 -> Result<Redirect, ErrorResponse> {
                    crate::models::delete_todo(&db, todo_id)?;

                    Ok(Redirect::see_other("/todos"))
                },
            )
            .post(
                "/todos/toggle/:todo_id",
                |State(db): State<KeyValueDatabase>,
                 Path(todo_id): Path<u64>|
                 -> Result<Redirect, ErrorResponse> {
                    let mut todo = match crate::models::get_todo(&db, todo_id)? {
                        Some(x) => x,
                        None => return Err(ErrorStatusCode::NotFound.into()),
                    };

                    todo.is_done = !todo.is_done;
                    crate::models::update_todo(&db, todo)?;

                    Ok(Redirect::see_other("/todos"))
                },
            )
    }

    fn todos_routes() -> Scope {
        Scope::new()
            .get("/", |State(db): State<KeyValueDatabase>, auth: AuthenticatedUser| -> Result<HTMLElement, ErrorResponse> {
                let AuthenticatedUser(user) = &auth;
                let todos = crate::models::get_all_todos(&db, user.id)?;

                Ok(html::html(|| {
                    Head(|| {
                        Title("TodoApp | Todos");
                        html::style(r#"
                         .animate-fade-in {
                            opacity: 0; 
                            transform: scale(0.95);
                            animation: fadeIn 300ms ease-in forwards;
                        }

                         @keyframes fadeIn {
                                to {
                                    transform: scale(1);
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
                                    if todos.is_empty() {
                                        html::div(|| {
                                            html::class("flex flex-row items-center justify-center");
                                            html::p(|| {
                                                html::content("No todos");
                                                html::class("font-bold text-gray-400 text-xl");
                                            });
                                        });
                                    }

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
                                                            html::class("p-2 bg-green-500 text-white rounded hover:bg-green-600 transition w-full"); 
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
            .get("/edit/:todo_id", |State(db): State<KeyValueDatabase>, auth: AuthenticatedUser, Path(todo_id): Path<u64>| -> Result<HTMLElement, ErrorResponse> {
                let todo = match crate::models::get_todo(&db, todo_id)? {
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
            .get("/:todo_id", |State(db): State<KeyValueDatabase>, auth: AuthenticatedUser, Path(todo_id): Path<u64>| -> Result<HTMLElement, ErrorResponse> {
                let todo = match crate::models::get_todo(&db, todo_id)? {
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

    fn home_routes() -> Scope {
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
                                            html::attr("autocomplete", "username");
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

    pub fn not_found() -> NotFound<HTMLElement> {
        NotFound(html::html(|| {
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
        }))
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
            html::link(|| {
                html::attr("rel", "icon");
                html::attr("href", "/favicon.png");
            });

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
            html::div(|| {
                html::class("w-full h-16");
            });

            html::header(|| {
                html::class("w-full h-16 shadow fixed top-0 left-0 flex flex-row p-2 justify-between items-center");
                
                html::a(|| {
                    html::attr("href", "/");
                    html::h4(|| {

                        html::content("TodoApp");
                        html::class("text-xl font-bold text-blue-500");
                    });
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
                else {
                    html::div("");
                }
            });

            content.into_children();
        })
    }
}

mod models {
    use std::{fmt::Display, time::Duration};
    use datetime::DateTime;
    use http1::error::BoxError;
    use http1_web::impl_serde_struct;
    use crate::kv::KeyValueDatabase;

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
        pub created_at: DateTime
    }

    impl_serde_struct!(Session => {
        id: String,
        user_id: u64,
        created_at: DateTime
   });

   const SESSION_DURATION: Duration = Duration::from_secs(60 * 60);

    #[derive(Debug)]
    pub struct ValidationError { field: &'static str, message: String }
    impl ValidationError {
        pub fn new(field: &'static str, message: impl Into<String>) -> Self {
            ValidationError { field, message: message.into()}
        }
    }

    impl std::error::Error for ValidationError {}

    impl Display for ValidationError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}: {}", self.field, self.message)
        }
    }

    pub trait Validate: Sized {
        fn validate_in_place(&mut self) -> Result<(), ValidationError>;
        fn validate(mut self) -> Result<Self, ValidationError> {
            self.validate_in_place()?;
            Ok(self)
        }
    }

    impl Validate for User {
        fn validate_in_place(&mut self) -> Result<(), ValidationError> {
            if self.username.trim().is_empty() {
                return Err(ValidationError::new("username", "cannot be empty"));
            }

            self.username = self.username.trim().into();

            Ok(())
        }
    }

    impl Validate for Todo {
        fn validate_in_place(&mut self) -> Result<(), ValidationError> {
            if self.title.trim().is_empty() {
                return Err(ValidationError::new("title", "cannot be empty"));
            }

            if let Some(description) = self.description.as_deref().filter(|x| x.is_empty()) {
                if description.trim().is_empty() {
                    return Err(ValidationError::new("description", "cannot be empty"));
                }
            }

            self.title = self.title.trim().into();
            self.description = self.description.as_deref().map(|x| x.trim().into());

            Ok(())
        }
    }


    pub fn insert_user(db: &KeyValueDatabase, username: String) -> Result<User, BoxError> {
        if get_user_by_username(db, &username)?.is_some() {
            return Err(String::from("user already exists").into())
        }

        let id = db.incr("next_user_id")? + 1;
        let user = User { id, username }.validate()?;
        db.set(format!("user/{id}"), user.clone())?;
        Ok(user)
    }

    pub fn get_user(db: &KeyValueDatabase, user_id: u64) -> Result<Option<User>, BoxError> {
        let key = &format!("user/{user_id}");
        let user = db.get::<User>(key)?;
        Ok(user)
    }

    pub fn get_user_by_username(db: &KeyValueDatabase, username: &str) -> Result<Option<User>, BoxError>  {
        let user = db.scan::<User>("user/")?.into_iter().find(|x| x.username == username);
        Ok(user)
    }

    pub fn insert_todo(
        db: &KeyValueDatabase,
        title: String,
        description: Option<String>,
        user_id: u64,
    ) -> Result<Todo, BoxError> {
        let id = db.incr("next_todo_id")? + 1;
        let todo = Todo { id, title, description, user_id, is_done: false  }.validate()?;
        db.set(format!("todo/{id}"), todo.clone())?;
        Ok(todo)
    }

    pub fn update_todo(db: &KeyValueDatabase, mut todo: Todo) -> Result<Option<Todo>, BoxError> {
        let key = &format!("todo/{}", todo.id);

        if db.contains(key)? {
            todo.validate_in_place()?;
            db.set(key, todo.clone())?;
            Ok(Some(todo))
        }
        else {
            Ok(None)
        }
    }

    pub fn delete_todo(db: &KeyValueDatabase, id: u64) -> Result<Option<Todo>, BoxError> {
        let key = &format!("todo/{id}");

        match db.get(key)? {
            Some(deleted) => {
                db.del(key)?;
                Ok(Some(deleted))
            },
            None => Ok(None),
        }
    }

    pub fn get_todo(db: &KeyValueDatabase, todo_id: u64) -> Result<Option<Todo>, BoxError> {
        let key = &format!("todo/{todo_id}");
        let user = db.get::<Todo>(key)?;
        Ok(user)
    }

    pub fn get_all_todos(db: &KeyValueDatabase, user_id: u64) -> Result<Vec<Todo>, BoxError> {
        let todos = db.scan::<Todo>("todo/")?
            .into_iter()
            .filter(|x| x.user_id == user_id)
            .collect::<Vec<_>>();

        Ok(todos)
    }

    pub fn create_session(db: &KeyValueDatabase, user_id: u64) -> Result<Session, BoxError> {
        let id = rng::sequence::<rng::random::Alphanumeric>()
            .take(32)
            .collect::<String>();

        let session = Session {
            id: id.clone(),
            user_id,
            created_at: DateTime::now_utc()
        };

        db.set(format!("session/{id}"), session.clone())?;
        Ok(session)
    }

    pub fn get_session_by_id(db: &KeyValueDatabase, session_id: String) -> Result<Option<Session>, BoxError> {
        remove_expired_sessions(db);

        let key = format!("session/{session_id}");
        let session = db.get::<Session>(key)?;
        Ok(session)
    }

    pub fn get_session_user(db: &KeyValueDatabase, session_id: String) -> Result<Option<User>, BoxError> {
        let session: Session = match get_session_by_id(db, session_id)? {
            Some(s) => s,
            None => return Ok(None),
        };

        get_user(db, session.user_id)
    }

    pub fn remove_session(db: &KeyValueDatabase, session_id: String) -> Result<(), BoxError> {
        remove_expired_sessions(db);

        let key = format!("session/{session_id}");
        db.del(key)?;
        Ok(())
    }

    pub fn remove_expired_sessions(db: &KeyValueDatabase)  {
       fn try_remove_expired_sessions(db: &KeyValueDatabase)  -> Result<(), BoxError>{
        let now = DateTime::now_utc();
        let expired_sessions = db.scan::<Session>("session/")?
            .into_iter()
            .filter(|s|  now > (s.created_at + SESSION_DURATION))
            .collect::<Vec<_>>();

        let deleted = db.retain(|key| {
            expired_sessions.iter().find(|s| s.id == key).is_some()
        })?;

        if deleted > 0 {
            log::debug!("`{deleted}` sessions deleted");
        }

        Ok(())
       }
    
       if let Err(err) = try_remove_expired_sessions(db) {
        log::error!("Failed to remove expired sessions: {err}");
       }
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

    #[derive(Debug, Clone)]
    pub struct KeyValueDatabase(PathBuf); 

    impl KeyValueDatabase {
        pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
            let cwd = std::env::current_dir()?;
            let file_path = cwd.join(path);

            if !file_path.exists() {
                let mut ancestors  = file_path.ancestors();
                ancestors.next();

                if let Some(dir) = ancestors.next() {
                    std::fs::create_dir_all(dir)?;
                }

                std::fs::write(&file_path, "{}")?;
                log::debug!("Created kv database file: {file_path:?}");
            }
            else {
                log::debug!("kv database file exists: {file_path:?}");
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

        pub fn set<T: Serialize>(&self, key: impl AsRef<str>, value: T) -> std::io::Result<()> {
            self.tap(|json| {
                let new_value = http1_web::serde::json::to_value(&value).map_err(|err| std::io::Error::other(err))?;
                json.try_insert(key.as_ref(), new_value).map_err(|err| std::io::Error::other(err))?;
                Ok(())
            })
        }

        pub fn get<T: Deserialize + 'static>(&self, key: impl AsRef<str>) -> std::io::Result<Option<T>> {
            self.tap(|json | {
                let value = match json.get(key.as_ref()) {
                    Some(x) => x,
                    None => return Ok(None),
                };

                let value = http1_web::serde::json::from_value::<T>(value.clone()).map_err(|err| std::io::Error::other(err))?;
                Ok(Some(value))
            })
        }

        pub fn scan<T: Deserialize>(&self, pattern: impl AsRef<str>) -> std::io::Result<Vec<T>> {
            self.tap(|json | {
                let pattern = pattern.as_ref();
                let mut values = Vec::new();

                match json {
                    JsonValue::Object(ordered_map) => {
                        for (k, v) in ordered_map.iter() {
                            if !k.starts_with(pattern) {
                                continue;
                            }

                            match http1_web::serde::json::from_value::<T>(v.clone()) {
                                Ok(x) => values.push(x),
                                Err(err) => {
                                    log::warn!("failed to scan value as `{}`: {err}", std::any::type_name::<T>());
                                },
                            };
                        }
                    },
                    v => panic!("expected json object but was `{}`", v.variant())
                }

                Ok(values)
            })
        }

        pub fn incr(&self, key: impl AsRef<str>) -> std::io::Result<u64> {
            self.tap(|json| {
                let key = key.as_ref();
                match json.get(key) {
                    Some(x) => {
                        if !x.is_number() {
                            return Err(std::io::Error::other(format!("`{key}` is not a number")));
                        }

                        let value = x.as_number().unwrap().as_u64().unwrap_or(0) + 1;
                        let new_value = JsonValue::from(value);
                        json.try_insert(key, new_value).map_err(|err| std::io::Error::other(err))?;
                        Ok(value)
                    },
                    None => {
                        json.try_insert(key, JsonValue::from(0)).map_err(|err| std::io::Error::other(err))?;
                        Ok(0)
                    },
                }
            })
        }

        pub fn contains(&self, key: impl AsRef<str>) -> std::io::Result<bool> {
            self.tap(|json | {
                match json.get(key.as_ref()) {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            })
        }

        pub fn del(&self, key: impl AsRef<str>) -> std::io::Result<bool> {
            self.tap(|json| {
                let deleted = json.remove(key.as_ref()).is_some();
                Ok(deleted)
            })
        }

        pub fn retain(&self, f: impl Fn(&str) -> bool) -> std::io::Result<usize> {
            self.tap(|json| {
                let mut deleted_count = 0;

                match json {
                    JsonValue::Object(ordered_map) => {
                        ordered_map.retain(|k, _| {
                            let should_remove = !f(k);
                            
                            if should_remove {
                                deleted_count += 1;
                            }

                            should_remove
                        });
                    },
                    v => panic!("expected json object but was `{}`", v.variant())
                }

                Ok(deleted_count)
            })
        }
    }
}