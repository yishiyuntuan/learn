//! Service is a special Component that supports dependency injection at compile time
// #![doc = include_str!("../../DI.md")]
use crate::application::AppBuilder;
use crate::component::ComponentRegistry;
use crate::config::ConfigRegistry;
use crate::error::Result;

pub use inventory::submit;

pub use springboot_macros::Service;

/// Service is a special Component that can inject dependent Components as field members
/// ```rust
/// use spring::plugin::service::Service;
/// use spring_sqlx::ConnectPool;
///
/// #[derive(Clone, Service)]
/// struct UserService {
///     #[inject(component)]
///     db: ConnectPool
/// }
/// ```
pub trait Service: Clone + Sized + 'static {
    /// Construct the Service component
    fn build<R>(registry: &R) -> Result<Self>
    where
        R: ComponentRegistry + ConfigRegistry;
}

//////////////////////////////////////////////////
/// Install the Service component into the App
pub trait ServiceRegistrar: Send + Sync + 'static {
    /// Install the Service component into the App
    fn install_service(&self, app: &mut AppBuilder) -> Result<()>;
}

inventory::collect!(&'static dyn ServiceRegistrar);

/// auto_config
#[macro_export]
macro_rules! submit_service {
    ($ty:ident) => {
        ::springboot::component::service::submit! {
            &$ty as &dyn ::springboot::component::service::ServiceRegistrar
        }
    };
}

/// Find all ServiceRegistrar and install them into the app
pub fn auto_inject_service(app: &mut AppBuilder) -> Result<()> {
    for registrar in inventory::iter::<&dyn ServiceRegistrar> {
        registrar.install_service(app)?;
    }
    Ok(())
}
