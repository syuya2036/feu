#[cfg(all(feature = "hyper", feature = "json"))]
mod app {
    use feu_adapter_hyper::serve;
    use feu_core::app::App;
    use feu_core::types::{FeuBody, FeuResponse};
    use feu_middleware::logger::Logger;
    use serde::{Deserialize, Serialize};
    use std::sync::{Arc, Mutex};

    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Todo {
        id: u64,
        title: String,
        completed: bool,
    }

    #[derive(Deserialize)]
    struct CreateTodo {
        title: String,
    }

    #[derive(Clone)]
    struct AppState {
        todos: Arc<Mutex<Vec<Todo>>>,
    }

    #[tokio::main]
    pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize tracing
        feu_core::tracing::init_tracing();

        let state = AppState {
            todos: Arc::new(Mutex::new(Vec::new())),
        };

        let mut app: App<AppState> = App::new();

        use feu_middleware::pretty_json::PrettyJson;
        // Register middleware
        app.use_mw(Logger::new());
        app.use_mw(PrettyJson::new());

        // Register routes
        app.get("/todos", handlers::list_todos);
        app.post("/todos", handlers::create_todo);

        let set_addr = "127.0.0.1:3000";
        println!("Listening on http://{}", set_addr);

        // Serve app
        serve(app, state, set_addr).await?;

        Ok(())
    }

    mod handlers {
        use super::*;
        use feu_core::ctx::Ctx;
        use feu_core::error::Result;

        pub async fn list_todos(ctx: Ctx<AppState>) -> Result<FeuResponse> {
            let todos = ctx.env.todos.lock().unwrap().clone();
            let json = serde_json::to_string(&todos).unwrap();

            Ok(FeuResponse::text(json).with_header(
                http::header::CONTENT_TYPE,
                http::header::HeaderValue::from_static("application/json"),
            ))
        }

        pub async fn create_todo(mut ctx: Ctx<AppState>) -> Result<FeuResponse> {
            // Simple body parsing (assuming text/json body is small and we read it all)
            // Ctx body is FeuBody.
            // Feu-core doesn't have built-in JSON body parser in Ctx yet?
            // We can extract bytes/text.

            let body_bytes = match ctx.req.body_mut() {
                FeuBody::Bytes(b) => b.clone(),
                FeuBody::Text(s) => bytes::Bytes::from(s.clone()),
                #[allow(unreachable_patterns)]
                _ => return Ok(FeuResponse::empty(http::StatusCode::BAD_REQUEST)),
            };

            let payload: CreateTodo = match serde_json::from_slice(&body_bytes) {
                Ok(p) => p,
                Err(_) => return Ok(FeuResponse::empty(http::StatusCode::BAD_REQUEST)),
            };

            let mut todos = ctx.env.todos.lock().unwrap();
            let id = todos.len() as u64 + 1;
            let todo = Todo {
                id,
                title: payload.title,
                completed: false,
            };
            todos.push(todo.clone());

            let json = serde_json::to_string(&todo).unwrap();
            Ok(FeuResponse::text(json)
                .with_status(http::StatusCode::CREATED)
                .with_header(
                    http::header::CONTENT_TYPE,
                    http::header::HeaderValue::from_static("application/json"),
                ))
        }
    }
}

pub fn main() {
    #[cfg(all(feature = "hyper", feature = "json"))]
    app::main().unwrap();

    #[cfg(not(all(feature = "hyper", feature = "json")))]
    println!("This example requires 'hyper' and 'json' features.");
}
