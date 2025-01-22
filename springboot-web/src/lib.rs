pub mod config;
/// springboot-web defined error
pub mod error;
/// axum extract
pub mod extractor;
/// axum route handler
pub mod handler;
pub mod middleware;

pub use axum;
pub use springboot::async_trait;
/////////////////web-macros/////////////////////
pub use springboot_macros::delete;
pub use springboot_macros::get;
pub use springboot_macros::head;
pub use springboot_macros::options;
pub use springboot_macros::patch;
pub use springboot_macros::post;
pub use springboot_macros::put;
/// To use these Procedural Macros, you need to add `springboot-web` dependency
pub use springboot_macros::route;
pub use springboot_macros::routes;
pub use springboot_macros::scope;
pub use springboot_macros::trace;

/// axum::routing::MethodFilter re-export
pub use axum::routing::MethodFilter;
/// MethodRouter with AppState
pub use axum::routing::MethodRouter;
/// Router with AppState
pub use axum::Router;


use anyhow::Context;
use axum::Extension;
use config::ServerConfig;
use config::WebConfig;
use springboot::component::component::ComponentRef;
use springboot::component::ComponentRegistry;
use springboot::component::MutableComponentRegistry;
use springboot::{
    application::{App, AppBuilder},
    component::Starter,
    config::ConfigRegistry,
    error::Result,
};
use std::{net::SocketAddr, ops::Deref, sync::Arc};

pub type Routers = Vec<Router>;
/// Web Configurator
pub trait WebConfigurator {
    /// add route to app registry
    fn add_router(&mut self, router: Router) -> &mut Self;
}

impl WebConfigurator for AppBuilder {
    fn add_router(&mut self, router: Router) -> &mut Self {
        if let Some(routers) = self.get_component_ref::<Routers>() {
            unsafe {
                let raw_ptr = ComponentRef::into_raw(routers);
                let routers = &mut *(raw_ptr as *mut Vec<Router>);
                routers.push(router);
            }
            self
        } else {
            self.add_component(vec![router])
        }
    }
}

/// State of App
#[derive(Clone)]
pub struct AppState {
    /// App Registry Ref
    pub app: Arc<App>,
}

/// Web Component Definition
pub struct WebStarter;

#[async_trait]
impl Starter for WebStarter {
    fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<WebConfig>()
            .expect("web component config load failed");

        // 1. collect router
        let routers = app.get_component_ref::<Routers>();
        let mut router: Router = match routers {
            Some(rs) => {
                let mut router = Router::new();
                for r in rs.deref().iter() {
                    router = router.merge(r.to_owned());
                }
                router
            }
            None => Router::new(),
        };
        if let Some(middlewares) = config.middlewares {
            router = middleware::apply_middleware(router, middlewares);
        }

        let server_conf = config.server;

        app.add_scheduler(move |app: Arc<App>| Box::new(Self::schedule(router, app, server_conf)));
    }
}

impl WebStarter {
    async fn schedule(router: Router, app: Arc<App>, config: ServerConfig) -> Result<String> {
        // 2. bind tcp listener
        let addr = SocketAddr::from((config.binding, config.port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("bind tcp listener failed:{}", addr))?;

        if addr.ip().is_unspecified() {
            let local_ip = local_ip_address::local_ip().unwrap();
            tracing::info!("bind tcp listener: http://{}:{}", local_ip,config.port);
            tracing::info!("bind tcp listener: http://127.0.0.1:{}",config.port );
            tracing::info!("bind tcp listener: http://localhost:{}", config.port);
        } else if addr.ip().is_loopback() {
            tracing::info!("bind tcp listener: http://127.0.0.1:{}",config.port );
            tracing::info!("bind tcp listener: http://localhost:{}", config.port);
        } else {
            tracing::info!("bind tcp listener: http://{addr}" );
        }

        // 3. axum server
        let router = router.layer(Extension(AppState { app }));

        tracing::info!("axum server started");
        if config.connect_info {
            // with client connect info
            let service = router.into_make_service_with_connect_info::<SocketAddr>();
            let server = axum::serve(listener, service);
            if config.graceful {
                server.with_graceful_shutdown(shutdown_signal()).await
            } else {
                server.await
            }
        } else {
            let service = router.into_make_service();
            let server = axum::serve(listener, service);
            if config.graceful {
                server.with_graceful_shutdown(shutdown_signal()).await
            } else {
                server.await
            }
        }
            .context("start axum server failed")?;

        Ok("axum schedule finished".to_string())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, waiting for web server shutdown")
        },
        _ = terminate => {
            tracing::info!("Received kill signal, waiting for web server shutdown")
        },
    }
}
