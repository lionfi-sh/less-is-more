use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use deadpool_postgres::Pool;
use fly::apis::configuration::Configuration as FlyClient;
use serde::{Deserialize, Serialize};
use thruster::{
    context::context_ext::ContextExt,
    errors::{ErrorSet, ThrusterError},
    Context, ContextState, MiddlewareNext, MiddlewareResult,
};

use crate::{
    app::{ClonableCtx, Ctx},
    errors::Error,
    models::{NonSecureUser, User},
};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CreateUser {
    email: String,
    password: String,
}

#[thruster::json_request]
pub(crate) async fn create_user(
    create_user: CreateUser,
    mut context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let CreateUser { email, password } = create_user;
    let db: &Pool = context.extra.get();
    let mut db = db.get().await.unwrap();
    let db = db.transaction().await.unwrap();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user: NonSecureUser = User::create(&db, email, password_hash)
        .await
        .unwrap()
        .into();

    #[cfg(not(test))]
    {
        let fly: &FlyClient = context.extra.get();
        crate::services::fly::create_app(fly, &user.id.to_string())
            .await
            .map_err(|e| {
                tracing::error!("An error occurred while calling fly.io to create an app: {e:#?}");
                ThrusterError::generic_error(context.clone_ctx())
            })?;
    }

    db.commit().await.unwrap();

    context.json(&user).map_err(|_e| {
        Error::GenericError(
            context.clone(),
            "Serialization error".to_string(),
            serde_json::Value::default(),
        )
        .into()
    })?;

    context.status(201);

    Ok(context)
}

#[thruster::middleware]
pub(crate) async fn get_user(
    mut context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let db: &Pool = context.extra.get();
    let user: &Option<User> = context.extra.get();
    let user: NonSecureUser = User::read(
        &db.get().await.unwrap(),
        &user.as_ref().map(|v| v.id.clone()).unwrap(),
    )
    .await
    .unwrap()
    .into();

    context.json(&user).map_err(|_e| {
        Error::GenericError(
            context.clone_ctx(),
            "Serialization error".to_string(),
            serde_json::Value::default(),
        )
        .into()
    })?;

    context.status(200);

    Ok(context)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        controllers::sessions::tests::create_user_and_session_helper,
        thruster_extensions::TestResponseExt,
    };
    use rand::distributions::DistString;
    use thruster::Testable;
    use uuid::Uuid;

    #[derive(Clone, Debug)]
    pub(crate) struct TestUser {
        pub(crate) email: String,
        pub(crate) password: String,
        pub(crate) id: Uuid,
    }

    pub(crate) async fn create_user_helper(app: &impl Testable) -> TestUser {
        let email = format!(
            "test-{}@lionfi.sh",
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        );
        let password = "abceasyas123".to_string();

        let non_secure_user = app
            .post(
                "/users",
                vec![],
                serde_json::to_vec(&CreateUser {
                    email: email.clone(),
                    password: password.clone(),
                })
                .unwrap(),
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(201, "It should have a created status")
            .json::<NonSecureUser>();

        assert_eq!(non_secure_user.email, email);

        TestUser {
            email,
            password,
            id: non_secure_user.id,
        }
    }

    #[tokio::test]
    async fn create_user_should_work() {
        let test_app = crate::app::init().await.commit();

        let _ = create_user_helper(&test_app).await;
    }

    #[tokio::test]
    async fn get_identity_should_return_the_current_users_identity() {
        let test_app = crate::app::init().await.commit();

        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let non_secure_user = (&test_app as &dyn Testable)
            .get(
                "/users",
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(200, "It should have an OK status")
            .json::<NonSecureUser>();

        assert_eq!(
            test_user.email, non_secure_user.email,
            "It should return the same user"
        );
    }

    #[tokio::test]
    async fn get_identity_should_require_a_valid_session() {
        let test_app = crate::app::init().await.commit();

        let (_test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = (&test_app as &dyn Testable)
            .get(
                "/users",
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}z", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(401, "It should have an unauthorized status");
    }
}
