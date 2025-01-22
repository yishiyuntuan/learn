mod common;
mod controller;
mod dao;
mod model;
mod services;

use springboot::{auto_config, App};
use springboot_web::WebConfigurator;
use springboot_web::WebStarter;

#[auto_config(WebConfigurator)] // 自动扫描web router
#[tokio::main]
async fn main() {
    App::new().logger(springboot_logger::LoggerStarter).add_starter(WebStarter).run().await
}
