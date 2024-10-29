use deadpool_postgres::Pool;
use fly::{
    apis::configuration::Configuration as FlyClient,
    models::{
        FlyPeriodMachineConfig, FlyPeriodMachineGuest, FlyPeriodMachineMount, FlyPeriodMachinePort,
        FlyPeriodMachineService,
    },
};
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
    models::{Image, ImageVersion, Job, JobStatus, User},
    services::fly::create_machine,
};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CreateJob {
    image_id: Uuid,
    image_version_id: Uuid,
    cpu: String,
    gpu: String,
}

#[thruster::json_request]
pub(crate) async fn create_job(
    create_job: CreateJob,
    mut context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let CreateJob {
        image_id,
        image_version_id,
        cpu,
        gpu,
    } = create_job;
    let user: &Option<User> = context.extra.get();
    let user = user.as_ref().unwrap();
    let db: &Pool = context.extra.get();
    let mut db = db.get().await.unwrap();
    let db = db.transaction().await.unwrap();

    let image = Image::read(&db, &image_id).await.map_err(|e| {
        tracing::error!("An error occurred while fetching an image: {e:#?}");
        ThrusterError::generic_error(context.clone_ctx())
    })?;

    if image.user_id != user.id {
        tracing::error!("User does not own image");
        return Err(ThrusterError::unauthorized_error(context));
    }

    let image_version = ImageVersion::read_where_image_id(&db, &image_id)
        .await
        .map_err(|e| {
            tracing::error!("An error occurred while fetching an image: {e:#?}");
            ThrusterError::generic_error(context.clone_ctx())
        })?
        .into_iter()
        .find(|v| v.id == image_version_id)
        .ok_or_else(|| ThrusterError::not_found_error(context.clone_ctx()))?;

    let job = Job::create(&db, user.id, JobStatus::Pending, image_version.id)
        .await
        .map_err(|e| {
            tracing::error!("An error occurred while creating a job: {e:#?}");
            ThrusterError::generic_error(context.clone_ctx())
        })?;

    #[cfg(not(test))]
    {
        let fly: &FlyClient = context.extra.get();
        create_machine(
            fly,
            &user.id.to_string(),
            &cpu,
            &gpu,
            &image,
            &image_version,
        )
        .await
        .map_err(|e| {
            tracing::error!("An error occurred while calling fly.io to create a machine: {e:#?}");
            ThrusterError::generic_error(context.clone_ctx())
        })?;
    }

    db.commit().await.unwrap();

    context.json(&job).map_err(|_e| {
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
pub(crate) async fn get_jobs(
    mut context: Ctx,
    _next: MiddlewareNext<Ctx>,
) -> MiddlewareResult<Ctx> {
    let db: &Pool = context.extra.get();
    let user: &Option<User> = context.extra.get();
    let jobs = Job::read_where_user_id(&db.get().await.unwrap(), &user.as_ref().unwrap().id)
        .await
        .map_err(|e| {
            tracing::error!("An error occurred while fetching images: {e:#?}");
            ThrusterError::generic_error(context.clone_ctx())
        })?;

    context.json(&jobs).map_err(|_e| {
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
        controllers::{
            images::tests::create_image_helper, sessions::tests::create_user_and_session_helper,
        },
        thruster_extensions::TestResponseExt,
    };
    use thruster::Testable;

    pub(crate) async fn create_job_helper(
        app: &impl Testable,
        user_id: &Uuid,
        session_token: &str,
    ) -> Job {
        let image = create_image_helper(app, user_id, session_token).await;

        let image_versions = app
            .get(
                &format!("/images/{}/versions", image.id),
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {session_token}"),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(200, "It should have an ok status")
            .json::<Vec<ImageVersion>>();

        app.post(
            "/jobs",
            vec![(
                "Authorization".to_string(),
                format!("Bearer {session_token}"),
            )],
            serde_json::to_vec(&CreateJob {
                image_id: image.id,
                image_version_id: image_versions.get(0).unwrap().id,
            })
            .unwrap(),
        )
        .await
        .expect("Should correctly resolve")
        .expect_status(201, "It should have a created status")
        .json::<Job>()
    }

    #[tokio::test]
    async fn create_job_should_work() {
        let test_app = crate::app::init().await.commit();

        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = create_job_helper(&test_app, &test_user.id, &session.token).await;
    }

    #[tokio::test]
    async fn get_jobs_should_work() {
        let test_app = crate::app::init().await.commit();

        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = create_job_helper(&test_app, &test_user.id, &session.token).await;

        let jobs = (&test_app as &dyn Testable)
            .get(
                "/jobs",
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(200, "It should have an OK status")
            .json::<Vec<Job>>();

        assert_eq!(jobs.len(), 1, "It should have a job");
    }

    #[tokio::test]
    async fn get_jobs_should_retrieve_for_a_single_user() {
        let test_app = crate::app::init().await.commit();

        // First user
        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = create_job_helper(&test_app, &test_user.id, &session.token).await;

        // Second user
        let (test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = create_job_helper(&test_app, &test_user.id, &session.token).await;

        let jobs = (&test_app as &dyn Testable)
            .get(
                "/jobs",
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}", session.token),
                )],
            )
            .await
            .expect("Should correctly resolve")
            .expect_status(200, "It should have an OK status")
            .json::<Vec<Job>>();

        assert_eq!(jobs.len(), 1, "It should have a single job");
    }

    #[tokio::test]
    async fn get_jobs_should_require_a_valid_session() {
        let test_app = crate::app::init().await.commit();

        let (_test_user, session) = create_user_and_session_helper(&test_app).await;
        let _ = (&test_app as &dyn Testable)
            .get(
                "/jobs",
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
