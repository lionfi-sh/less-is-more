use std::str::FromStr;

use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use thruster::{
    context::context_ext::ContextExt,
    errors::{ErrorSet, ThrusterError},
    Context, ContextState, MiddlewareNext, MiddlewareResult,
};
use uuid::Uuid;

use crate::{
    app::{ClonableCtx, Ctx},
    errors::Error,
    models::{Image, ImageVersion, User},
};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CreateImage {
    nickname: String,
    image_url: String,
}

#[thruster::json_request]
pub(crate) async fn create_image(
    create_image: CreateImage,
    mut context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let CreateImage {
        nickname,
        image_url,
    } = create_image;
    let user: &Option<User> = context.extra.get();
    let user = user.as_ref().unwrap();
    let db: &Pool = context.extra.get();
    let db = db.get().await.unwrap();
    let image = Image::create(&db, user.id, nickname, image_url)
        .await
        .map_err(|e| {
            tracing::error!("An error occurred while creating an image: {e:#?}");
            ThrusterError::generic_error(context.clone_ctx())
        })?;
    let _image_version = ImageVersion::create(&db, image.id, "".to_string(), "latest".to_string())
        .await
        .map_err(|e| {
            tracing::error!("An error occurred while creating an image version: {e:#?}");
            ThrusterError::generic_error(context.clone_ctx())
        })?;

    context.json(&image).map_err(|_e| {
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
pub(crate) async fn get_images(
    mut context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let db: &Pool = context.extra.get();
    let user: &Option<User> = context.extra.get();
    let images = Image::read_where_user_id(&db.get().await.unwrap(), &user.as_ref().unwrap().id)
        .await
        .map_err(|e| {
            tracing::error!("An error occurred while fetching images: {e:#?}");
            ThrusterError::generic_error(context.clone_ctx())
        })?;

    context.json(&images).map_err(|_e| {
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

#[thruster::middleware]
pub(crate) async fn get_image_versions(
    mut context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let db: &Pool = context.extra.get();
    let db = db.get().await.unwrap();

    let user: &Option<User> = context.extra.get();
    let image_id = Uuid::from_str(&context.params().get("id").unwrap().param).map_err(|e| {
        tracing::error!("Invalid image id format: {e:#?}");
        ThrusterError::generic_error(context.clone_ctx())
    })?;

    let image = Image::read(&db, &image_id).await.map_err(|e| {
        tracing::error!("Could not load image: {e:#?}");
        ThrusterError::generic_error(context.clone_ctx())
    })?;

    if image.user_id != user.as_ref().unwrap().id {
        return Err(ThrusterError::unauthorized_error(context));
    }

    let image_versions = ImageVersion::read_where_image_id(&db, &image_id)
        .await
        .map_err(|e| {
            tracing::error!("An error occurred while fetching images: {e:#?}");
            ThrusterError::generic_error(context.clone_ctx())
        })?;

    context.json(&image_versions).map_err(|_e| {
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

    pub(crate) async fn create_image_helper(
        app: &impl Testable,
        _user_id: &Uuid,
        session_token: &str,
    ) -> Image {
        let nickname = format!(
            "test-{}",
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        );
        let image_url = format!(
            "https://registry.lionfi.sh/images/test-{}",
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        );

        app.post(
            "/images",
            vec![(
                "Authorization".to_string(),
                format!("Bearer {session_token}"),
            )],
            serde_json::to_vec(&CreateImage {
                nickname,
                image_url,
            })
            .unwrap(),
        )
        .await
        .expect("Should correctly resolve")
        .expect_status(201, "It should have a created status")
        .json::<Image>()
    }

    #[tokio::test]
    async fn create_image_should_work() {
        let test_app = crate::app::init().await.commit();

        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = create_image_helper(&test_app, &test_user.id, &session.token).await;
    }

    #[tokio::test]
    async fn get_images_should_work() {
        let test_app = crate::app::init().await.commit();

        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = create_image_helper(&test_app, &test_user.id, &session.token).await;

        let images = (&test_app as &dyn Testable)
            .get(
                "/images",
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(200, "It should have an OK status")
            .json::<Vec<Image>>();

        assert_eq!(images.len(), 1, "It should have a image");
    }

    #[tokio::test]
    async fn get_images_should_retrieve_for_a_single_user() {
        let test_app = crate::app::init().await.commit();

        // First user
        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = create_image_helper(&test_app, &test_user.id, &session.token).await;

        // Second user
        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = create_image_helper(&test_app, &test_user.id, &session.token).await;

        let images = (&test_app as &dyn Testable)
            .get(
                "/images",
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(200, "It should have an OK status")
            .json::<Vec<Image>>();

        assert_eq!(images.len(), 1, "It should have a single image");
    }

    #[tokio::test]
    async fn get_images_should_require_a_valid_session() {
        let test_app = crate::app::init().await.commit();

        let (_test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = (&test_app as &dyn Testable)
            .get(
                "/images",
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}z", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(401, "It should have an unauthorized status");
    }

    #[tokio::test]
    async fn get_image_version_should_work() {
        let test_app = crate::app::init().await.commit();

        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let image = create_image_helper(&test_app, &test_user.id, &session.token).await;

        let image_versions = (&test_app as &dyn Testable)
            .get(
                &format!("/images/{}/versions", image.id),
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(200, "It should have an OK status")
            .json::<Vec<ImageVersion>>();

        assert_eq!(image_versions.len(), 1, "It should have an image version");
    }

    #[tokio::test]
    async fn get_image_versions_should_require_a_valid_session() {
        let test_app = crate::app::init().await.commit();

        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let image = create_image_helper(&test_app, &test_user.id, &session.token).await;

        let _ = (&test_app as &dyn Testable)
            .get(
                &format!("/images/{}/versions", image.id),
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}z", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(401, "It should have an unauthorized status");
    }

    #[tokio::test]
    async fn get_image_versions_should_require_a_valid_session_from_the_owning_user() {
        let test_app = crate::app::init().await.commit();

        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let image = create_image_helper(&test_app, &test_user.id, &session.token).await;
        let (_test_user, session) = create_user_and_session_helper(&test_app).await;

        let _ = (&test_app as &dyn Testable)
            .get(
                &format!("/images/{}/versions", image.id),
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(401, "It should have an unauthorized status");
    }
}
