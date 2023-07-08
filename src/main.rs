use actix_web::{web, App, HttpServer};
use mobc::Pool;
use mobc_redis::redis;
use mobc_redis::RedisConnectionManager;
use settings::Settings;
use std::time::Duration;

mod api;
mod settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let s = Settings::new().unwrap(); // initialize settings
    let ip = s.server.get_ip();

    //initialize redis connection pool
    let client = redis::Client::open(s.redis.url).unwrap();
    let manager = RedisConnectionManager::new(client);
    let pool: Pool<RedisConnectionManager> = Pool::builder()
        .get_timeout(Some(Duration::from_secs(s.redis.pool_timeout_secs)))
        .max_idle(s.redis.pool_max_idle)
        .max_open(s.redis.pool_max_open)
        .build(manager);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(s.code.clone()))
            .configure(api::links::init_routes)
    })
    .workers(s.server.worker as usize)
    .bind(&ip)?
    .run()
    .await
}
