use crate::models::InsertableUser;
use axum::{
    extract::{multipart::Field, Json, Multipart},
    http::StatusCode,
    Extension,
};
use axum_macros::debug_handler;
use eyre::ErrReport;
use eyre::{Context, ContextCompat};
use std::collections::HashMap;
use tracing::warn;

use crate::{
    erro::{add_error, merge_errors, AppError},
    models::{InsertableUserBuilder, User},
    repositories::{EmailRepository, ImageRepository, UserRepository},
    tokens,
};

async fn parse_username(field: Field<'_>) -> Result<String, ErrReport> {
    field.text().await.wrap_err("failed to parse form username")
}

async fn parse_password(field: Field<'_>) -> Result<String, ErrReport> {
    field.text().await.wrap_err("failed to parse form password")
}

async fn parse_full_name(field: Field<'_>) -> Result<String, ErrReport> {
    field
        .text()
        .await
        .wrap_err("failed to parse form full name")
}

async fn parse_email(field: Field<'_>) -> Result<String, ErrReport> {
    field.text().await.wrap_err("failed to parse form email")
}

async fn parse_image_content_type(field: &Field<'_>) -> Result<String, ErrReport> {
    field
        .content_type()
        .wrap_err("failed to fetch image content type")
        .map(str::to_string)
}

async fn parse_image(field: Field<'_>) -> Result<axum::body::Bytes, ErrReport> {
    field.bytes().await.wrap_err("failed to parse form image")
}

/// Register a user.
#[debug_handler]
pub async fn register(
    mut body: Multipart,
    Extension(user_repository): Extension<UserRepository>,
    Extension(email_repository): Extension<EmailRepository>,
    Extension(image_repository): Extension<ImageRepository>,
    Extension(redis_client): Extension<redis::Client>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let mut builder = InsertableUserBuilder::new();
    let mut errors = HashMap::<String, Vec<String>>::new();

    while let Some(field) = body
        .next_field()
        .await
        .wrap_err("failed to parse multipart form data")?
    {
        if let Some(field_name) = field.name() {
            match field_name {
                "username" => {
                    builder = builder.with_username(parse_username(field).await?);
                }
                "password" => {
                    builder = builder.with_password(parse_password(field).await?);
                }
                "full_name" => {
                    builder = builder.with_full_name(parse_full_name(field).await?);
                }
                "email" => {
                    builder = builder.with_email_id(
                        email_repository
                            .create_email(parse_email(field).await?)
                            .await?,
                    );
                }
                "profile_pic" => {
                    let mime_type = parse_image_content_type(&field).await?;
                    let image = parse_image(field).await?;
                    builder = builder.with_profile_pic_id(
                        image_repository
                            .create_image(&mime_type, image.to_vec())
                            .await?,
                    );
                }
                _ => {
                    errors = add_error(errors, field_name.to_string(), "Invalid field".to_string());
                    warn!("invalid field name in registration form: {}", field_name);
                }
            }
        }
    }

    // TODO: insert permissions for new user

    match builder.build() {
        Ok(insertable_user) => send_user(insertable_user, user_repository, redis_client).await,
        Err(errs) => Err(AppError::UnprocessableEntity(merge_errors(errors, errs))),
    }
}

async fn send_user(
    user: InsertableUser,
    user_repository: UserRepository,
    redis_client: redis::Client,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = user_repository.create_user(user).await?;

    let _confirmation_token = tokens::new_expirable_token(
        redis_client,
        format!("confirmation_tokens:{}", &user.username),
        60 * 10,
    )
    .await?;

    // TODO: send token in confirmation email

    Ok((StatusCode::CREATED, Json(user)))
}
