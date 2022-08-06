use eyre::Context;
use rand::rngs::OsRng;
use rand::RngCore;
use redis::AsyncCommands;

use crate::erro::AppError;

/// Return a base32 encoded random generated token of the given size.
fn generate_token<const N: usize>() -> String {
    let mut token = [0u8; N];
    OsRng.fill_bytes(&mut token);
    base32::encode(base32::Alphabet::Crockford, &token)
}

/// Create a new token that expires `seconds` in the future associated to `subject`.
pub async fn new_expirable_token<T: ToString>(
    redis_client: redis::Client,
    subject: T,
    seconds: usize,
) -> Result<String, AppError> {
    let token = generate_token::<32>();

    // TODO: I can't find a way to pass a reference to this around.
    let mut conn = redis_client
        .get_async_connection()
        .await
        .wrap_err("failed to connect to redis")?;

    conn.set_ex(subject.to_string(), &token, seconds)
        .await
        .wrap_err("failed to create token in redis")?;

    Ok(token)
}
