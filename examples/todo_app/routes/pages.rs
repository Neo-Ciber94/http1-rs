use http1_web::{
    app::Scope,
    html::{self, element::HTMLElement},
    path::Path,
    redirect::Redirect,
    state::State,
    ErrorResponse, ErrorStatusCode, NotFound,
};

use crate::{
    components::{AlertProps, Head, Layout, LayoutProps, Title},
    db::KeyValueDatabase,
};

use super::models::AuthenticatedUser;

pub fn page_routes() -> Scope {
    Scope::new()
        .scope("/", home_routes())
        .scope("/todos", todos_routes())
        .fallback(not_found)
}

fn todos_routes() -> Scope {
    Scope::new()
        .get("/", |State(db): State<KeyValueDatabase>, auth: AuthenticatedUser, alert: Option<AlertProps>| -> Result<HTMLElement, ErrorResponse> {
            let AuthenticatedUser(user) = &auth;
            let todos = crate::models::get_all_todos(&db, user.id)?;

            Ok(html::html(|| {
                Head(|| {
                    Title("TodoApp | Todos");
                });
            
                html::body(|| {
                    Layout(LayoutProps { auth: Some(auth), alert }, || {
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
        .get("/create", | auth: AuthenticatedUser, alert: Option<AlertProps>| {
            html::html(|| {
                Head(|| {
                    Title("TodoApp | Create Todo");
                });

                html::body(|| {
                    Layout(LayoutProps { auth: Some(auth), alert }, || {
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
        .get("/edit/:todo_id", |State(db): State<KeyValueDatabase>, auth: AuthenticatedUser, Path(todo_id): Path<u64>, alert: Option<AlertProps>| -> Result<HTMLElement, ErrorResponse> {
            let todo = match crate::models::get_todo(&db, todo_id)? {
                Some(x) => x,
                    None => {
                    return Err(ErrorStatusCode::NotFound.into())
                }
            };

            dbg!(&todo);
            Ok(html::html(|| {
                Head(|| {
                    Title("TodoApp | Edit Todo");
                });

                html::body(|| {
                    Layout(LayoutProps { auth: Some(auth), alert }, || {
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
                                        html::class("mt-4 p-3 border border-gray-300 rounded w-full");
                                        html::content( todo.description.clone().unwrap_or_default());
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
        .get("/:todo_id", |State(db): State<KeyValueDatabase>, auth: AuthenticatedUser, Path(todo_id): Path<u64>, alert: Option<AlertProps>| -> Result<HTMLElement, ErrorResponse> {
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
                    Layout(LayoutProps { auth: Some(auth), alert }, || {
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
        .get("/login", |auth: Option<AuthenticatedUser>, alert: Option<AlertProps>| -> Result<HTMLElement, Redirect> {
            if auth.is_some() {
                return Err(Redirect::see_other("/todos"));
            }

            Ok(html::html(|| {
                Head(|| {
                    Title("TodoApp | Login");
                });

                html::body(|| {
                    Layout(LayoutProps { auth, alert }, || {
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

fn not_found() -> NotFound<HTMLElement> {
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
