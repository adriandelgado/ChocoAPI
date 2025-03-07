use http_api_problem::StatusCode;
use reqwest::multipart;

use chocoapi::models::User;

use crate::helpers::TestApp;

#[tokio::test]
async fn hitting_register_with_valid_data_returns_created_and_new_user_as_json() {
    // Arrange
    let app = TestApp::new().await;
    let client = reqwest::Client::new();
    let form_data = multipart::Form::new()
        .text("username", "johndoe")
        .text("password", "12345")
        .text("full_name", "John Doe")
        .text("email", "john@doe.com");

    // Act
    let response = client
        .post(&format!("{}/register", &app.address))
        .multipart(form_data)
        .send()
        .await
        .expect("failed to execute request");

    let response_status = response.status();
    let created_user: User = response
        .json()
        .await
        .expect("failed to parse user from server response");

    // Assert
    assert!(response_status.is_success());
    assert!(created_user.active);
    assert_eq!(Some("John Doe".to_string()), created_user.full_name);
    assert_eq!("johndoe".to_string(), created_user.username);

    let db_user = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users
         WHERE username = $1
        "#,
        created_user.username
    )
    .fetch_one(&*app.db)
    .await
    .expect("failed to retrieve user from databse");

    assert_eq!(db_user.username, "johndoe");
}

#[tokio::test]
async fn hitting_register_endpoint_with_missing_username_returns_unprocessable_entity() {
    // Arrange
    let app = TestApp::new().await;
    let client = reqwest::Client::new();
    let form_data = multipart::Form::new()
        .text("password", "12345")
        .text("email", "john@doe.com");

    // Act
    let response = client
        .post(&format!("{}/register", &app.address))
        .multipart(form_data)
        .send()
        .await
        .expect("failed to execute request");

    let response_status = response.status();
    let created_user: Result<User, _> = response.json().await;

    // Assert
    assert!(created_user.is_err());
    assert!(response_status.is_client_error());
    assert_eq!(StatusCode::UNPROCESSABLE_ENTITY, response_status);
}
