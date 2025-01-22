use crate::Router;
pub use inventory::submit;

pub trait HandlerRegistrar: Send + Sync + 'static {
    /// 注册处理方法到路由, Router::new().route("create", get(create_user));
    fn register(&self, router: Router) -> Router;
}

inventory::collect!(&'static dyn HandlerRegistrar);

/// auto_config
#[macro_export]
macro_rules! submit_typed_handler {
    ($ty:ident) => {
        ::springboot_web::handler::submit! {
            &$ty as &dyn ::springboot_web::handler::HandlerRegistrar
        }
    };
}

// auto_config
pub fn auto_router() -> Router {
    let mut router = Router::new();
    for handler in inventory::iter::<&dyn HandlerRegistrar> {
        router = handler.register(router);
    }
    router
}
