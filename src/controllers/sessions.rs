use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::{engine::general_purpose, Engine};
use deadpool_postgres::Pool;
use rand::RngCore;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use thruster::{
    context::context_ext::ContextExt,
    errors::{ErrorSet, ThrusterError},
    middleware::cookies::{Cookie, CookieOptions, HasCookies, SameSite},
    Context, ContextState, MiddlewareNext, MiddlewareResult,
};
use uuid::Uuid;

use crate::{
    app::{ClonableCtx, Ctx},
    errors::Error,
    models::User,
};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CreateSessionRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SessionResponse {
    pub(crate) token: String,
}

#[thruster::json_request]
pub(crate) async fn create_session(
    create_session: CreateSessionRequest,
    mut context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let CreateSessionRequest { email, password } = create_session;
    let db: &Pool = context.extra.get();

    let user = User::read_by_email(&db.get().await.unwrap(), &email)
        .await
        .map_err(|e| {
            tracing::error!("Unable to access user: {email}\n\n{e:#?}");
            ThrusterError::unauthorized_error(context.clone_ctx())
        })?;

    Argon2::default()
        .verify_password(
            password.as_bytes(),
            &PasswordHash::new(&user.password_hash).unwrap(),
        )
        .map_err(|e| {
            tracing::error!("Invalid password for user: {email}\n\n{e:#?}");
            ThrusterError::unauthorized_error(context.clone_ctx())
        })?;

    let mut rand_bytes: [u8; 32] = [0; 32];
    rand::thread_rng().fill_bytes(&mut rand_bytes);
    let token = general_purpose::STANDARD.encode(&rand_bytes);

    let redis: &redis::Client = context.extra.get();
    let mut conn = redis.get_multiplexed_async_connection().await.unwrap();
    let session_expiration = std::env::var("SESSION_EXPIRATION")
        .unwrap_or_else(|_| format!("{}", 60 * 60 * 24 * 14 /* two weeks */))
        .parse::<u64>()
        .unwrap();

    let _: String = conn
        .set_ex(
            _session_key(&token),
            &user.id.to_string(),
            session_expiration,
        )
        .await
        .unwrap();

    context.cookie(
        "Authorization",
        &urlencoding::encode(&format!("Bearer {token}")),
        &CookieOptions {
            http_only: true,
            ..CookieOptions::default()
        },
    );

    context.json(&SessionResponse { token }).map_err(|_e| {
        Error::GenericError(
            context.clone_ctx(),
            "Serialization error".to_string(),
            serde_json::Value::default(),
        )
        .into()
    })?;

    context.status(201);

    Ok(context)
}

#[thruster::middleware]
pub(crate) async fn authenticate(
    mut context: Ctx,
    next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let token = &context
        .req_header("Authorization")
        .or_else(|| {
            context
                .cookies
                .get("Authorization")
                .map(|v| v.value.as_str())
        })
        .unwrap_or("       ")[7..];

    let redis: &redis::Client = context.extra.get();

    let mut conn = redis.get_multiplexed_async_connection().await.unwrap();
    let user_id: Option<Uuid> = conn
        .get(_session_key(token))
        .await
        .ok()
        .flatten()
        .and_then(|v: String| Uuid::parse_str(&v).ok());

    match user_id {
        Some(user_id) => {
            let db_user = {
                let db: &Pool = context.extra.get();
                User::read(&db.get().await.unwrap(), &user_id)
                    .await
                    .unwrap()
            };
            let user: &mut Option<User> = context.extra.get_mut();
            *user = Some(db_user);
            context = next(context).await?;
        }
        None => {
            return Err(ThrusterError::unauthorized_error(context.clone_ctx()));
        }
    }

    Ok(context)
}

fn _session_key(token: &str) -> String {
    format!("{token}:session")
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        controllers::users::tests::{create_user_helper, TestUser},
        thruster_extensions::TestResponseExt,
    };
    use thruster::Testable;

    pub(crate) async fn create_session_helper(
        app: &impl Testable,
        test_user: &TestUser,
    ) -> SessionResponse {
        let session = app
            .post(
                "/sessions",
                vec![],
                serde_json::to_vec(&CreateSessionRequest {
                    email: test_user.email.clone(),
                    password: test_user.password.clone(),
                })
                .unwrap(),
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(201, "It should have a created status")
            .json::<SessionResponse>();

        session
    }

    pub(crate) async fn create_user_and_session_helper(
        app: &impl Testable,
    ) -> (TestUser, SessionResponse) {
        let test_user = create_user_helper(app).await;
        let session = create_session_helper(app, &test_user).await;

        (test_user, session)
    }

    #[tokio::test]
    async fn create_session() {
        let test_app = crate::app::init().await.commit();

        let test_user = create_user_helper(&test_app).await;
        let _ = create_session_helper(&test_app, &test_user).await;
    }
}
