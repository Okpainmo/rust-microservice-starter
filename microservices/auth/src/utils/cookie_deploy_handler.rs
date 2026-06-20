use crate::utils::load_config::AppConfig;
use time;
use tower_cookies::{Cookie, Cookies};

pub async fn deploy_auth_cookie(cookies: Cookies, cookie_value: String, config: &AppConfig) {
    // let cookie = Cookie::build(("name", "value"))
    //     .domain("www.rustychat.com")
    //     .path("/")
    //     .secure(true)
    //     .http_only(true);
    //
    // jar.add(cookie);
    // // jar.remove(Cookie::build("name").path("/"));

    // Create a basic cookie
    let mut cookie = Cookie::new("auth_cookie", cookie_value);

    let auth = config
        .auth
        .as_ref()
        .expect("AUTH CONFIGURATION IS MISSING!");
    let is_dev = config.app.environment.as_deref().unwrap_or("production") == "development";

    // Set cookie attributes for security
    cookie.set_path("/");
    cookie.set_http_only(true);
    // Only set secure in non-development or if explicitly needed
    cookie.set_secure(!is_dev);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);

    // Optional: set expiration from config
    cookie.set_max_age(time::Duration::hours(
        auth.jwt_refresh_expiration_time_in_hours as i64,
    ));

    cookies.add(cookie);
}
