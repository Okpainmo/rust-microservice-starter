use crate::common::{authenticated_request, register_authenticated_user, setup_test_server};

#[tokio::test]
async fn test_list_roles_success() {
    let server = setup_test_server().await;
    let auth = register_authenticated_user(&server).await;

    authenticated_request(server.get("/api/v1/auth/roles"), &auth)
        .await
        .assert_status(axum::http::StatusCode::OK);
}
