use crate::common::resp::{Resp, Response};
use springboot_logger::{debug, error, info, warn};
// use springboot::
use springboot_web::get;

mod post_controller;


#[get("/")]
async fn index() -> Resp {
    info!("Hello World");
    debug!("Hello World");
    warn!("Hello World");
    error!("Hello World");
    Response::ok("Hello World")
}
