use crate::AppState;
use crate::core::controllers::login::login_user::login_user;
use crate::core::controllers::logout::logout_user::logout_user;
use crate::core::controllers::permissions::create_permission::create_permission_controller;
use crate::core::controllers::permissions::list_permissions::list_permissions;
use crate::core::controllers::permissions::list_user_permissions::list_user_permissions;
use crate::core::controllers::permissions::update_permission::update_permission_controller;
use crate::core::controllers::register::register_admin::register_admin;
use crate::core::controllers::register::register_user::register_user;
use crate::core::controllers::roles::assign_role_permission::assign_role_permission_controller;
use crate::core::controllers::roles::assign_user_role::assign_user_role_controller;
use crate::core::controllers::roles::create_role::create_role_controller;
use crate::core::controllers::roles::delete_role_permission::delete_role_permission_controller;
use crate::core::controllers::roles::list_roles::list_roles;
use crate::core::controllers::roles::list_user_roles::list_user_roles;
use crate::core::controllers::roles::remove_user_role::remove_user_role_controller;
use crate::core::controllers::roles::update_role::update_role_controller;
use crate::core::controllers::sessions::get_session::get_session;
use crate::core::controllers::sessions::list_sessions::list_sessions;
use crate::core::controllers::sessions::list_user_sessions::list_user_sessions;
use crate::core::controllers::sessions::update_session::update_session;
use crate::middlewares::access_middleware::access_middleware;
use crate::middlewares::sessions_middleware::sessions_middleware;
use axum::{
    Router, middleware,
    routing::{get, patch, post},
};
use tower_cookies::CookieManagerLayer;

pub fn auth_routes(state: &AppState) -> Router<AppState> {
    let protected_routes = Router::new()
        .route("/sessions", get(list_sessions))
        .route("/sessions/user/{user_id}", get(list_user_sessions))
        .route(
            "/sessions/{session_id}",
            get(get_session).patch(update_session),
        )
        .route("/roles", get(list_roles).post(create_role_controller))
        .route("/roles/{role_id}", patch(update_role_controller))
        .route(
            "/roles/permissions",
            post(assign_role_permission_controller).delete(delete_role_permission_controller),
        )
        .route("/roles/user/assign", post(assign_user_role_controller))
        .route("/roles/user/remove", post(remove_user_role_controller))
        .route("/roles/user/{user_id}", get(list_user_roles))
        .route("/logout", post(logout_user))
        .route(
            "/permissions",
            get(list_permissions).post(create_permission_controller),
        )
        .route(
            "/permissions/{permission_id}",
            patch(update_permission_controller),
        )
        .route("/permissions/user/{user_id}", get(list_user_permissions))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            access_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            sessions_middleware,
        ));

    Router::new()
        .route("/register", post(register_user))
        .route("/register/admin", post(register_admin))
        .route("/login", post(login_user))
        .merge(protected_routes)
        .layer(CookieManagerLayer::new())
}
