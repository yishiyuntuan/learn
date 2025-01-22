use crate::common::resp::{Resp, Response};
use crate::model::BlogPost;
use crate::services;
use crate::services::blog_service::query;
use springboot_logger::info;
use springboot_macros::{get, post};
use springboot_web::axum::Json;
use springboot_web::extractor::Path;

// 新增博客

#[post("/post")]
async fn post_upsert(Json(post): Json<BlogPost>) -> Resp {
    info!("{:?}", post);
    let p = services::blog_service::update_or_save(post).await?;
    Response::ok(p)
}

#[get("/post/{id}")]
async fn get_post(Path(id): Path<String>) -> Resp {
    let result = query(id).await?;
    Response::ok(result)
}
