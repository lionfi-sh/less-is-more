use std::{env, sync::atomic::AtomicUsize, time::Instant};

use deadpool_postgres::{Config, Pool, Runtime};
use fly::apis::configuration::Configuration as FlyClient;
use http::method::Method;
use redis::Client as RedisClient;
use thruster::{
    context::typed_hyper_context::TypedHyperContext, context_state, m, middleware_fn, App, Context,
    HyperRequest, MiddlewareNext, MiddlewareResult,
};
use tokio_postgres::NoTls;
use tracing::info;

use crate::{
    controllers::{
        images::{create_image, get_image_versions, get_images},
        jobs::{create_job, get_jobs},
        sessions::{authenticate, create_session},
        users::{create_user, get_user},
    },
    models::User,
};

#[context_state]
pub struct State(RequestCounter, Pool, RedisClient, Option<User>, FlyClient);

pub struct ServerConfig {
    db: Pool,
    cache: RedisClient,
    fly: FlyClient,
}

pub type Ctx = TypedHyperContext<State>;

#[derive(Default)]
pub struct RequestCounter(AtomicUsize);

pub(crate) trait ClonableCtx {
    fn clone_ctx(&self) -> Self;
}

impl ClonableCtx for Ctx {
    fn clone_ctx(&self) -> Self {
        let pool: &Pool = self.extra.get();
        let cache: &RedisClient = self.extra.get();
        let fly: &FlyClient = self.extra.get();
        Ctx::new_without_request(State(
            RequestCounter::default(),
            pool.clone(),
            cache.clone(),
            None,
            fly.clone(),
        ))
    }
}

fn generate_context(request: HyperRequest, state: &ServerConfig, _path: &str) -> Ctx {
    Ctx::new(
        request,
        State(
            RequestCounter::default(),
            state.db.clone(),
            state.cache.clone(),
            None,
            state.fly.clone(),
        ),
    )
}

#[middleware_fn]
async fn profiling(context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let start_time = Instant::now();

    let method = context
        .hyper_request
        .as_ref()
        .unwrap()
        .request
        .method()
        .clone();
    let path_and_query = context
        .hyper_request
        .as_ref()
        .unwrap()
        .request
        .uri()
        .path_and_query()
        .unwrap()
        .clone();

    let result = next(context).await;
    let context_ref = match &result {
        Ok(context) => &context,
        Err(e) => &e.context,
    };

    let elapsed_time = start_time.elapsed();
    if path_and_query.path() != "/ping" {
        info!(
            "{}μs\t\t{}\t{}\t{}",
            elapsed_time.as_micros(),
            method,
            context_ref.status,
            path_and_query,
        );
    }

    result
}

#[middleware_fn]
async fn count(context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let counter: &RequestCounter = context.extra.get();
    counter.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    next(context).await
}

#[middleware_fn]
async fn cors(mut context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.set("Access-Control-Allow-Origin", "*");
    context.set("Access-Control-Allow-Headers", "*");
    context.set("Access-Control-Allow-Credentials", "true");

    let method = context
        .hyper_request
        .as_ref()
        .unwrap()
        .request
        .method()
        .clone();
    if &method.to_string() == "OPTIONS" {
        return Ok(context);
    }

    next(context).await
}

#[middleware_fn]
async fn identity(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    Ok(context)
}

#[middleware_fn]
async fn ping(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body("pong");

    Ok(context)
}

pub async fn generate_default_server_config() -> ServerConfig {
    let mut cfg = Config::default();
    cfg.dbname = env::var("POSTGRES_DB")
        .ok()
        .or_else(|| Some("limdb".to_string()));
    cfg.user = env::var("POSTGRES_USER")
        .ok()
        .or_else(|| Some("limadmin".to_string()));
    cfg.password = env::var("POSTGRES_PASSWORD")
        .ok()
        .or_else(|| Some("limpassword".to_string()));
    cfg.host = env::var("POSTGRES_HOST")
        .ok()
        .or_else(|| Some("localhost".to_string()));

    let db = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let cache = RedisClient::open(format!(
        "redis://{}",
        env::var("REDIS_URL")
            .ok()
            .or_else(|| Some("127.0.0.1".to_string()))
            .unwrap()
    ))
    .expect("Could not create a redis client");

    let mut fly = FlyClient::new();
    fly.bearer_access_token = env::var("FLY_API_TOKEN").ok();
    info!("Running fly with configuration: {fly:#?}");

    ServerConfig { db, cache, fly }
}

pub async fn init() -> App<HyperRequest, Ctx, ServerConfig> {
    info!("Initializing app...");

    init_with_config(generate_default_server_config().await).await
}

pub async fn init_with_config(server_config: ServerConfig) -> App<HyperRequest, Ctx, ServerConfig> {
    App::<HyperRequest, Ctx, ServerConfig>::create(generate_context, server_config)
        .middleware("/", m![profiling, count, cors])
        .get("/ping", m![ping])
        .post("/users", m![create_user])
        .get("/users", m![authenticate, get_user])
        .post("/sessions", m![create_session])
        .post("/images", m![authenticate, create_image])
        .get("/images", m![authenticate, get_images])
        .get("/images/:id/versions", m![authenticate, get_image_versions])
        .post("/jobs", m![authenticate, create_job])
        .get("/jobs", m![authenticate, get_jobs])
        .set404(m![identity])
}
