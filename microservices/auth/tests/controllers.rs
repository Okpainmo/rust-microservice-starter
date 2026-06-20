#[path = "common/shared.rs"]
mod common;

#[path = "controllers/login/login_user.rs"]
mod login_user;

#[path = "controllers/logout/logout_user.rs"]
mod logout_user;

#[path = "controllers/permissions/create_permission.rs"]
mod create_permission;

#[path = "controllers/permissions/list_permissions.rs"]
mod list_permissions;

#[path = "controllers/permissions/list_user_permissions.rs"]
mod list_user_permissions;

#[path = "controllers/permissions/update_permission.rs"]
mod update_permission;

#[path = "controllers/register/register_admin.rs"]
mod register_admin;

#[path = "controllers/register/register_user.rs"]
mod register_user;

#[path = "controllers/roles/assign_role_permission.rs"]
mod assign_role_permission;

#[path = "controllers/roles/assign_user_role.rs"]
mod assign_user_role;

#[path = "controllers/roles/create_role.rs"]
mod create_role;

#[path = "controllers/roles/delete_role_permission.rs"]
mod delete_role_permission;

#[path = "controllers/roles/list_roles.rs"]
mod list_roles;

#[path = "controllers/roles/list_user_roles.rs"]
mod list_user_roles;

#[path = "controllers/roles/remove_user_role.rs"]
mod remove_user_role;

#[path = "controllers/roles/update_role.rs"]
mod update_role;

#[path = "controllers/sessions/get_session.rs"]
mod get_session;

#[path = "controllers/sessions/list_sessions.rs"]
mod list_sessions;

#[path = "controllers/sessions/list_user_sessions.rs"]
mod list_user_sessions;

#[path = "controllers/sessions/update_session.rs"]
mod update_session;
