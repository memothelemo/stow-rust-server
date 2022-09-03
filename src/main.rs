use std::time::Duration;

use crate::common::ServerSession;
use actix_extensible_rate_limit::{
    backend::{memory::InMemoryBackend, SimpleInputFunctionBuilder},
    RateLimiter,
};
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use rbxcloud::rbx::{RbxCloud, UniverseId};

mod common;
mod routes;
mod services;

fn main() -> std::io::Result<()> {
    dotenv().unwrap();
    env_logger::builder()
        .parse_env(Env::new().default_filter_or("info"))
        .init();

    log::info!("running server...");

    let data: common::ActiveServerMap = Default::default();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.spawn(common::run_cleanup(data.clone()));

    let server = HttpServer::new(move || {
        let cloud = RbxCloud::new(
            &std::env::var("CLOUD_SECRET").expect("CLOUD_SECRET environment variable is required"),
            UniverseId(
                std::env::var("UNIVERSE_ID")
                    .expect("UNIVERSE_ID environment variable is required")
                    .parse()
                    .expect("not a valid number"),
            ),
        );

        let backend = InMemoryBackend::builder().build();
        let input = SimpleInputFunctionBuilder::new(Duration::from_secs(60), 50)
            .real_ip_key()
            .build();

        let middleware = RateLimiter::builder(backend, input).add_headers().build();

        App::new()
            .app_data(web::Data::new(ServerSession {
                active_servers: data.clone(),
                rbxcloud: cloud,
            }))
            .wrap(services::Auth)
            .wrap(middleware)
            .service(routes::register)
            .service(routes::logout)
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run();

    rt.block_on(server)
}
