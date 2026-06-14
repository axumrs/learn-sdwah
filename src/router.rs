use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};
use tower_http::services::ServeDir;

use crate::{ch01_jumping_in, ch03_developing_endpoints, ch04_recipes_for_common_scenarios};

pub fn init(state: ch03_developing_endpoints::Ch03ArcState) -> Router {
    Router::new()
        .nest("/api", api(state))
        .fallback_service(ServeDir::new("htmx_files"))
}

fn api(state: ch03_developing_endpoints::Ch03ArcState) -> Router {
    Router::new()
        .nest("/ch01", _init_ch01())
        .nest("/ch03", _init_ch03())
        .nest("/ch03-todo", _init_ch03_todo(state))
        .nest("/ch04", _init_ch04())
}

fn _init_ch01() -> Router {
    Router::new()
        .route("/version", get(ch01_jumping_in::version))
        .route("/dog", post(ch01_jumping_in::add_dog))
        .route("/table-rows", get(ch01_jumping_in::dog_rows))
        .route("/dog/{id}", delete(ch01_jumping_in::del))
}

fn _init_ch03() -> Router {
    Router::new()
        .route("/oob-demo", get(ch03_developing_endpoints::oob_demo))
        .route(
            "/event-with-no-data",
            get(ch03_developing_endpoints::event_with_no_data),
        )
        .route(
            "/event-with-string",
            get(ch03_developing_endpoints::event_with_string),
        )
        .route(
            "/event-with-object",
            get(ch03_developing_endpoints::event_with_object),
        )
        .route("/dog", post(ch03_developing_endpoints::add_dog))
        .route("/table-rows", get(ch03_developing_endpoints::dog_rows))
        .route(
            "/dog/{id}",
            delete(ch03_developing_endpoints::del_dog).put(ch03_developing_endpoints::edit_dog),
        )
        .route("/form", get(ch03_developing_endpoints::dog_form))
        .route(
            "/dog/select/{id}",
            put(ch03_developing_endpoints::select_dog),
        )
        .route(
            "/dog/deselect",
            put(ch03_developing_endpoints::deselect_dog),
        )
}

fn _init_ch03_todo(state: ch03_developing_endpoints::Ch03ArcState) -> Router {
    Router::new()
        .route(
            "/",
            post(ch03_developing_endpoints::add_todo).get(ch03_developing_endpoints::todo_list),
        )
        .route("/status", get(ch03_developing_endpoints::todo_status))
        .route(
            "/{id}",
            patch(ch03_developing_endpoints::edit_todo).delete(ch03_developing_endpoints::del_todo),
        )
        .route(
            "/completed/{id}/{completed}",
            patch(ch03_developing_endpoints::todo_completed),
        )
        .with_state(state)
}

fn _init_ch04() -> Router {
    Router::new()
        .route("/users", get(ch04_recipes_for_common_scenarios::user_list))
        .route(
            "/email-validate",
            get(ch04_recipes_for_common_scenarios::email_validate),
        )
        .route(
            "/password-validate",
            get(ch04_recipes_for_common_scenarios::password_validate),
        )
        .route(
            "/register-user",
            post(ch04_recipes_for_common_scenarios::register_user),
        )
        .route(
            "/search",
            get(ch04_recipes_for_common_scenarios::search)
                .post(ch04_recipes_for_common_scenarios::search),
        )
}
