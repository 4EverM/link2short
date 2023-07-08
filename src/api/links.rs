use actix_web::{
    get,
    http::header,
    post,
    web::{self, Json, Path, ServiceConfig},
    HttpResponse, Responder,
};
use anyhow::{anyhow, Result};
use mobc::Pool;
use mobc_redis::redis::{AsyncCommands, AsyncIter};
use mobc_redis::RedisConnectionManager;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api::ApiResult;
use crate::settings::Code;

// the key of hset to save link
const KEY: &str = "tinyCode";
// the default url if the tiny_code don't exist
const DEFAULT_URL: &str = "http://127.0.0.1:8080/";

// custom types
type RedisPool = web::Data<Pool<RedisConnectionManager>>;
type MobcConn = mobc::Connection<RedisConnectionManager>;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Link {
    tiny_code: String,
    origin_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ApiAddLInk {
    origin_url: String,
}

impl ApiAddLInk {
    fn to_short_link(self, code_len: u64) -> Link {
        let code_len = code_len as usize;
        Link {
            tiny_code: nanoid!(code_len),
            origin_url: self.origin_url,
        }
    }
}

#[get("/")]
/// index page
async fn index() -> impl Responder {
    "Welcome to the home page"
}

#[post("/create")]
/// reate a new tiny_link
async fn create_link(
    link: Json<ApiAddLInk>,
    pool: RedisPool,
    code: web::Data<Code>,
) -> impl Responder {
    let new_link: Link = link.0.to_short_link(code.length);
    if let Err(err) = insert2_redis(&new_link, pool).await {
        return Json(ApiResult::error(err.to_string()));
    }
    Json(ApiResult::success(Some(new_link.tiny_code)))
}

#[get("/{tiny_code}")]
/// returns the origin link for the given code
async fn get_from_link(path: Path<String>, pool: RedisPool) -> impl Responder {
    let tiny_code: String = path.into_inner();
    let mut conn: MobcConn = pool.get().await.unwrap();
    let target_url: String = match conn.hget(&KEY, &tiny_code).await.unwrap() {
        Some(v) => v,
        None => DEFAULT_URL.to_string(),
    };

    HttpResponse::Found()
        .append_header((header::LOCATION, target_url))
        .finish()
}

#[get("/links/all")]
async fn get_all_links(pool: RedisPool) -> impl Responder {
    let mut conn: MobcConn = pool.get().await.unwrap();
    let mut res: AsyncIter<(String, String)> = conn.hscan(&KEY).await.unwrap();

    let mut links_map: HashMap<String, String> = HashMap::new();
    while let Some(element) = res.next_item().await {
        links_map.insert(element.0, element.1);
    }

    Json(ApiResult::success(Some(links_map)))
}

/// init the routes
pub fn init_routes(cfg: &mut ServiceConfig) {
    cfg.service(index);
    cfg.service(create_link);
    cfg.service(get_from_link);
    cfg.service(get_all_links);
}

/// insert a link into the redis hset
async fn insert2_redis(link: &Link, pool: RedisPool) -> Result<()> {
    let mut conn: MobcConn = pool.get().await.unwrap();
    // check if the link already exists
    if conn.hexists(&KEY, &link.tiny_code).await.unwrap() {
        return Err(anyhow!("tiny_code already exists"));
    }

    let _: () = conn.hset(&KEY, &link.tiny_code, &link.origin_url).await?;
    Ok(())
}
