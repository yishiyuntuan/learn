mod auto;
mod config;
mod inject;
mod route;
mod scope;

use proc_macro::TokenStream;
use syn::DeriveInput;

fn input_and_compile_error(mut item: TokenStream, err: syn::Error) -> TokenStream {
    let compile_err = TokenStream::from(err.to_compile_error());
    item.extend(compile_err);
    item
}

// 生成属性宏route
/// Creates resource handler, allowing multiple HTTP method guards.
///
/// # Syntax
/// ```plain
/// #[route("path", method="HTTP_METHOD"[, attributes])]
/// ```
///
/// # Attributes
/// - `"path"`: Raw literal string with path for which to register handler.
/// - `method = "HTTP_METHOD"`: Registers HTTP method to provide guard for. Upper-case string,
///   "GET", "POST" for example.
///
/// # Examples
/// ```
/// # use springboot_web::axum::response::IntoResponse;
/// # use springboot_macros::route;
/// #[route("/test", method = "GET", method = "HEAD")]
/// async fn example() -> impl IntoResponse {
///     "hello world"
/// }
/// ```
#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    route::with_method(None, args, input)
}

/// Creates resource handler, allowing multiple HTTP methods and paths.
///
/// # Syntax
/// ```plain
/// #[routes]
/// #[<method>("path", ...)]
/// #[<method>("path", ...)]
/// ...
/// ```
///
/// # Attributes
/// The `routes` macro itself has no parameters, but allows specifying the attribute macros for
/// the multiple paths and/or methods, e.g. [`GET`](macro@get) and [`POST`](macro@post).
///
/// These helper attributes take the same parameters as the [single method handlers](crate#single-method-handler).
///
/// # Examples
/// ```
/// # use springboot_web::axum::response::IntoResponse;
/// # use springboot_macros::routes;
/// #[routes]
/// #[get("/test")]
/// #[get("/test2")]
/// #[delete("/test")]
/// async fn example() -> impl IntoResponse {
///     "hello world"
/// }
/// ```
#[proc_macro_attribute]
pub fn routes(_: TokenStream, input: TokenStream) -> TokenStream {
    route::with_methods(input)
}

macro_rules! method_macro {
    ($variant:ident, $method:ident) => {
        ///
        /// # Syntax
        /// ```plain
        #[doc = concat!("#[", stringify!($method), r#"("path"[, attributes])]"#)]
        /// ```
        ///
        /// # Attributes
        /// - `"path"`: Raw literal string with path for which to register handler.
        ///
        /// # Examples
        /// ```
        /// # use spring_web::axum::response::IntoResponse;
        #[doc = concat!("# use springboot_macros::", stringify!($method), ";")]
        #[doc = concat!("#[", stringify!($method), r#"("/")]"#)]
        /// async fn example() -> impl IntoResponse {
        ///     "hello world"
        /// }
        /// ```
        #[proc_macro_attribute]
        pub fn $method(args: TokenStream, input: TokenStream) -> TokenStream {
            route::with_method(Some(route::MethodType::$variant), args, input)
        }
    };
}

// 生成属性宏get post put delete head options trace patch
method_macro!(Get, get);
method_macro!(Post, post);
method_macro!(Put, put);
method_macro!(Delete, delete);
method_macro!(Head, head);
method_macro!(Options, options);
method_macro!(Trace, trace);
method_macro!(Patch, patch);

/// Prepends a path prefix to all handlers using routing macros inside the attached module.
///
/// # Syntax
///
/// ```
/// # use spring_macros::nest;
/// #[nest("/prefix")]
/// mod api {
///     // ...
/// }
/// ```
///
/// # Arguments
///
/// - `"/prefix"` - Raw literal string to be prefixed onto contained handlers' paths.
///
/// # Example
///
/// ```
/// # use spring_macros::{nest, get};
/// # use spring_web::axum::response::IntoResponse;
/// #[scope("/api")]
/// mod api {
///     # use super::*;
///     #[get("/hello")]
///     pub async fn hello() -> impl IntoResponse {
///         // this has path /api/hello
///         "Hello, world!"
///     }
/// }
/// # fn main() {}
/// ```
#[proc_macro_attribute]
pub fn scope(args: TokenStream, input: TokenStream) -> TokenStream {
    scope::with_scope(args, input)
}

/// Auto config
/// ```diff
///  use spring_macros::auto_config;
///  use spring_web::{WebPlugin, WebConfigurator};
///  use spring_job::{JobPlugin, JobConfigurator};
///  use spring_boot::app::App;
/// +#[auto_config(WebConfigurator, JobConfigurator)]
///  #[tokio::main]
///  async fn main() {
///      App::new()
///         .add_plugin(WebPlugin)
///         .add_plugin(JobPlugin)
/// -       .add_router(router())
/// -       .add_jobs(jobs())
///         .run()
///         .await
///  }
/// ```
///
#[proc_macro_attribute]
pub fn auto_config(args: TokenStream, input: TokenStream) -> TokenStream {
    auto::config(args, input)
}

/// Configurable
#[proc_macro_derive(Configurable, attributes(config_prefix))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    config::expand_derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Injectable Servcie
#[proc_macro_derive(Service, attributes(prototype, inject))]
pub fn derive_service(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    inject::expand_derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
