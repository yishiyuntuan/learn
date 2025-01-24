use std::collections::HashSet;

use crate::input_and_compile_error;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, LitStr, Path, Token};

macro_rules! standard_http_method {
    (
        $($variant:ident, $upper:ident, $lower:ident,)+
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum MethodType {
            $(
                $variant,
            )+
        }

        impl MethodType {

            #[allow(dead_code)]
            fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)+
                }
            }

            fn parse(method: &str) -> Result<Self, String> {
                match method {
                    $(stringify!($upper) => Ok(Self::$variant),)+
                    _ => Err(format!("HTTP method must be uppercase: `{}`", method)),
                }
            }

            pub(crate) fn from_path(method: &Path) -> Result<Self, ()> {

                // 获取路径的最后一个 segment
                if let Some(last_segment) = method.segments.last() {
                    // 比较最后一个 `ident` 是否匹配
                    match last_segment.ident.to_string().as_str() {
                                    $(stringify!($lower) => Ok(Self::$variant),)+
                        _ => Err(()),
                    }
                } else {
                    // 如果路径为空，返回错误
                    Err(())
                }
            }
        }

        impl ToTokens for MethodType {
            fn to_tokens(&self, output: &mut TokenStream2) {
                let stream = match self {
                    $(Self::$variant => quote!(::springboot_web::MethodFilter::$upper),)+
                };

                output.extend(stream);
            }
        }
    };
}

standard_http_method! {
    Get,       GET,     get,
    Post,      POST,    post,
    Put,       PUT,     put,
    Delete,    DELETE,  delete,
    Head,      HEAD,    head,
    Options,   OPTIONS, options,
    Trace,     TRACE,   trace,
    Patch,     PATCH,   patch,
}

impl TryFrom<&syn::LitStr> for MethodType {
    type Error = syn::Error;
    fn try_from(value: &syn::LitStr) -> Result<Self, Self::Error> {
        Self::parse(value.value().as_str())
            .map_err(|message| syn::Error::new_spanned(value, message))
    }
}

#[derive(Debug)]
struct Args {
    path: syn::LitStr,
    methods: HashSet<MethodType>,
}

impl Args {
    fn new(args: RouteArgs, method: Option<MethodType>) -> syn::Result<Self> {
        let mut methods = HashSet::new();

        let is_route_macro: bool = method.is_none();
        if let Some(method) = method {
            methods.insert(method);
        }

        for nv in args.options {
            if nv.path.is_ident("method") {
                if !is_route_macro {
                    return Err(syn::Error::new_spanned(
                        &nv,
                        "HTTP method forbidden here; to handle multiple methods, use `route` instead",
                    ));
                } else if let syn::Expr::Lit(syn::ExprLit {
                                                 lit: syn::Lit::Str(lit),
                                                 ..
                                             }) = nv.value.clone()
                {
                    if !methods.insert(MethodType::try_from(&lit)?) {
                        return Err(syn::Error::new_spanned(
                            nv.value,
                            format!("HTTP method defined more than once: `{}`", lit.value()),
                        ));
                    }
                } else {
                    return Err(syn::Error::new_spanned(
                        nv.value,
                        "Attribute method expects literal string",
                    ));
                }
            } else {
                return Err(syn::Error::new_spanned(
                    nv.path,
                    "Unknown attribute key is specified; allowed: method",
                ));
            }
        }

        Ok(Args {
            path: args.path,
            methods,
        })
    }
}

#[derive(Debug)]
pub(crate) struct RouteArgs {
    pub(crate) path: LitStr,
    pub(crate) options: Punctuated<syn::MetaNameValue, Token![,]>,
}

impl syn::parse::Parse for RouteArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        // path to match: "/foo"
        let path = input.parse::<syn::LitStr>().map_err(|mut err| {
            err.combine(syn::Error::new(
                err.span(),
                r#"invalid route definition, expected #[<method>("<path>")]"#,
            ));

            err
        })?;

        // verify that path pattern is valid
        // let _ = ResourceDef::new(path.value());

        // if there's no comma, assume that no options are provided
        if !input.peek(Token![,]) {
            return Ok(Self {
                path,
                options: Punctuated::new(),
            });
        }

        // advance past comma separator
        input.parse::<Token![,]>()?;

        // if next char is a literal, assume that it is a string and show multi-path error
        if input.cursor().literal().is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                r#"Multiple paths specified! There should be only one."#,
            ));
        }

        // zero or more options: name = "foo"
        let options = input.parse_terminated(syn::MetaNameValue::parse, Token![,])?;

        Ok(Self { path, options })
    }
}

struct Route {
    /// Name of the handler function being annotated.
    name: syn::Ident,

    /// Args passed to routing macro.
    ///
    /// When using `#[routes]`, this will contain args for each specific routing macro.
    args: Vec<Args>,

    /// AST of the handler function being annotated.
    ast: syn::ItemFn,

    /// The doc comment attributes to copy to generated struct, if any.
    doc_attributes: Vec<syn::Attribute>,
}

impl Route {
    pub fn new(args: RouteArgs, ast: syn::ItemFn, method: Option<MethodType>) -> syn::Result<Self> {
        let name = ast.sig.ident.clone();

        // Try and pull out the doc comments so that we can reapply them to the generated struct.
        // Note that multi line doc comments are converted to multiple doc attributes.
        let doc_attributes = ast
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .cloned()
            .collect();

        let args = Args::new(args, method)?;

        if args.methods.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "The #[route(..)] macro requires at least one `method` attribute",
            ));
        }

        if matches!(ast.sig.output, syn::ReturnType::Default) {
            return Err(syn::Error::new_spanned(
                ast,
                "Function has no return type. Cannot be used as handler",
            ));
        }

        Ok(Self {
            name,
            args: vec![args],
            ast,
            doc_attributes,
        })
    }

    fn multiple(args: Vec<Args>, ast: syn::ItemFn) -> syn::Result<Self> {
        let name = ast.sig.ident.clone();

        // Try and pull out the doc comments so that we can reapply them to the generated struct.
        // Note that multi line doc comments are converted to multiple doc attributes.
        let doc_attributes = ast
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .cloned()
            .collect();

        if matches!(ast.sig.output, syn::ReturnType::Default) {
            return Err(syn::Error::new_spanned(
                ast,
                "Function has no return type. Cannot be used as handler",
            ));
        }

        Ok(Self {
            name,
            args,
            ast,
            doc_attributes,
        })
    }
}

impl ToTokens for Route {
    fn to_tokens(&self, output: &mut TokenStream2) {
        let Self {
            name,
            ast,
            args,
            doc_attributes,
        } = self;

        #[allow(unused_variables)] // used when force-pub feature is disabled
        let vis = &ast.vis;

        let registrations: TokenStream2 = args
            .iter()
            .map(|args| {
                let Args { path, methods } = args;
                let method_binder = methods
                    .iter()
                    .map(|m| quote! {
                        let __method_router=::springboot_web::MethodRouter::on(__method_router, #m, #name);
                        }
                    );

                quote! {
                    let __method_router = ::springboot_web::MethodRouter::new();
                    #(#method_binder)*
                     __router = ::springboot_web::Router::route(__router, #path, __method_router);
                }
            })
            .collect();
        let stream = quote! {
            #(#doc_attributes)*
            #[allow(non_camel_case_types, missing_docs)]
            #vis struct #name;

            impl ::springboot_web::handler::HandlerRegistrar for #name {
                fn register(&self, mut __router: ::springboot_web::Router) -> ::springboot_web::Router{
                    #ast
                    #registrations
                    __router

                }
            }

            ::springboot_web::submit_typed_handler!(#name);
        };

        output.extend(stream);
    }
}

pub(crate) fn with_method(
    method: Option<MethodType>,
    args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let args = match syn::parse(args) {
        Ok(args) => args,
        // on parse error, make IDEs happy; see fn docs
        Err(err) => return input_and_compile_error(input, err),
    };

    let ast = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(ast) => ast,
        // on parse error, make IDEs happy; see fn docs
        Err(err) => return input_and_compile_error(input, err),
    };

    match Route::new(args, ast, method) {
        Ok(route) => route.into_token_stream().into(),
        // on macro related error, make IDEs happy; see fn docs
        Err(err) => input_and_compile_error(input, err),
    }
}

pub(crate) fn with_methods(input: TokenStream) -> TokenStream {
    let mut ast = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(ast) => ast,
        // on parse error, make IDEs happy; see fn docs
        Err(err) => return input_and_compile_error(input, err),
    };
    let (methods, others) = ast
        .attrs
        .into_iter()
        .map(|attr| match MethodType::from_path(attr.path()) {
            Ok(method) => Ok((method, attr)),
            Err(_) => Err(attr),
        })
        .partition::<Vec<_>, _>(Result::is_ok);

    ast.attrs = others.into_iter().map(Result::unwrap_err).collect();

    let methods = match methods
        .into_iter()
        .map(Result::unwrap)
        .map(|(method, attr)| {
            attr.parse_args()
                .and_then(|args| Args::new(args, Some(method)))
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(methods) if methods.is_empty() => {
            return input_and_compile_error(
                input,
                syn::Error::new(
                    Span::call_site(),
                    "The #[routes] macro requires at least one `#[<method>(..)]` attribute.",
                ),
            )
        }
        Ok(methods) => methods,
        Err(err) => return input_and_compile_error(input, err),
    };

    match Route::multiple(methods, ast) {
        Ok(route) => route.into_token_stream().into(),
        // on macro related error, make IDEs happy; see fn docs
        Err(err) => input_and_compile_error(input, err),
    }
}
