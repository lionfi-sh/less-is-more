use fly::{
    apis::{configuration::Configuration as FlyClient, Error},
    models::{
        FlyPeriodMachineConfig, FlyPeriodMachineGuest, FlyPeriodMachineMount, FlyPeriodMachinePort,
        FlyPeriodMachineService, Machine,
    },
};

use crate::models::{Image, ImageVersion};

pub async fn create_app(fly: &FlyClient, app_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(fly::apis::apps_api::apps_create(
        &fly,
        fly::models::CreateAppRequest {
            app_name: Some(app_id.to_string()),
            enable_subdomains: None,
            network: None,
            // TODO(trezm): Make this into an env var
            org_slug: Some("peter-mertz".to_string()),
        },
    )
    .await?)
}

pub async fn create_machine(
    fly: &FlyClient,
    app_id: &str,
    cpu: &str,
    gpu: &str,
    image: &Image,
    image_version: &ImageVersion,
) -> Result<Machine, Box<dyn std::error::Error>> {
    Ok(fly::apis::machines_api::machines_create(
        &fly,
        app_id,
        fly::models::CreateMachineRequest {
            region: Some("ord".to_string()),
            config: Some(Box::new(FlyPeriodMachineConfig {
                files: Some(vec![]),
                image: Some(format!(
                    "{}:{}",
                    image.image_url, image_version.version_number
                )),
                mounts: None,
                // mounts: Some(vec![FlyPeriodMachineMount {
                //     path: Some("/app/repositories".to_string()),
                //     size_gb: Some(20),
                //     volume: Some("repositories".to_string()),
                //     ..Default::default()
                // }]),
                services: Some(vec![FlyPeriodMachineService {
                    autostart: Some(true),
                    autostop: Some(true),
                    internal_port: Some(8888),
                    min_machines_running: Some(0),
                    ports: Some(vec![FlyPeriodMachinePort {
                        force_https: Some(false),
                        handlers: Some(vec!["http".to_string()]),
                        port: Some(8080),
                        ..Default::default()
                    }]),
                    protocol: Some("tcp".to_string()),
                    ..Default::default()
                }]),
                guest: Some(Box::new(FlyPeriodMachineGuest {
                    cpus: Some(4),
                    cpu_kind: Some(cpu.to_string()),
                    gpu_kind: Some(gpu.to_string()),
                    memory_mb: Some(1024 * 16),
                    ..Default::default()
                })),
                ..Default::default()
            })),
            ..Default::default()
        },
    )
    .await?)
}
