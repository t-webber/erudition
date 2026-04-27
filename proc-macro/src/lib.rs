//! Proc-macro crate to create proc-macros for erudition.

use std::sync::{Mutex, MutexGuard};

use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, FnArg, Ident, ItemFn, LitStr, Pat, parse};

thread_local! {
    static ROUTES: Mutex<Vec<String>> = Mutex::default();
}

/// Wrapper to create proc-macros for the request handling routines from
/// `actix_web` (e.g. `get`, `post`, `put`, etc.).
macro_rules! macros {
    ($($request:ident),*) => {
        $(
            #[proc_macro_attribute]
            #[doc = concat!("Wrapper of the ", stringify!($request), " actix request, with automatic registration through `routes!` and automatic logging")]
            pub fn $request(arg: TokenStream, body: TokenStream) -> TokenStream {
            logged_actix_request(&quote!($request).into(), arg, body)
            }
        )*
    };
}

/// Gives a mutable pointer to the [`ROUTES`] static list of registered routes.
fn get_routes<T, F: Fn(&mut MutexGuard<'_, Vec<String>>) -> T>(mtd: F) -> T {
    ROUTES.with(|routes| match routes.lock() {
        Ok(mut vec) => mtd(&mut vec),
        Err(mut poison) => mtd(poison.get_mut()),
    })
}

macros!(get, post, put);

/// Fires a nice compilation error when the macro is not used correctly.
fn error(attr: impl Into<proc_macro2::TokenStream>, msg: &str) -> TokenStream {
    Error::new_spanned(attr.into(), msg).to_compile_error().into()
}

/// Wrapper to create proc-macros for the request handling routines from
/// `actix_web` (e.g. `get`, `post`, `put`, etc.).
///
/// # Panics
///
/// If the proc-macro is used incorrectly.
fn logged_actix_request(
    request: &proc_macro2::TokenStream,
    arg: TokenStream,
    body: TokenStream,
) -> TokenStream {
    let Ok(path): Result<LitStr, _> = parse(arg.clone()) else {
        return error(arg, "expected #[this(\"/path\")]");
    };
    let Ok(function): Result<ItemFn, _> = parse(body.clone()) else {
        return error(body, "attribute only works on a function");
    };

    let block = &function.block;
    let sig = &function.sig;
    let name = &function.sig.ident;

    let args = match function
        .sig
        .inputs
        .iter()
        .filter_map(log_param)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(error) => return error,
    };

    get_routes(|routes| routes.push(name.to_string()));

    quote! {
        #[allow(clippy::literal_string_with_formatting_args)]
        #[actix_web::#request(#path)]
        #sig {
            let mut log = String::new();
            #(#args)*
            let res = HttpResponse::from({ #block });
            state.log(&format!("\x1b[36m{:4} {} ({}){}\x1b[0m", stringify!(#request), res.status().as_u16(), #path, &log));
            res
        }
    }.into()
}

/// Adds the value of a parameter to the logs.
fn log_param(
    input: &FnArg,
) -> Option<Result<proc_macro2::TokenStream, TokenStream>> {
    if let syn::FnArg::Typed(pat_type) = input
        && let Pat::Ident(pat_ident) = *pat_type.pat.clone()
    {
        let ident = pat_ident.ident;
        let string = ident.to_string();
        if string == "state" {
            return None;
        }
        Some(Ok(quote!(
            log.push_str(&format!(" [{}: {:?}]",
                #string,
                &#ident
            ));
        )))
    } else {
        Some(Err(error(quote!(#input), "invalid function argument")))
    }
}

/// Registers all the routes at once, to not forget to register them once
/// defined.
///
/// # Panics
///
/// If the proc-macro is used incorrectly.
#[proc_macro]
pub fn routes(app: TokenStream) -> TokenStream {
    let Ok(ident): Result<Ident, _> = parse(app.clone()) else {
        return error(app, "argument should be an identifier");
    };

    let stmts: Vec<_> = get_routes(|routes| {
        routes
            .iter()
            .map(|name| {
                let function = Ident::new(name, proc_macro2::Span::call_site());
                quote!(#function)
            })
            .map(|id| quote!(.service(#id)))
            .collect()
    });

    let expanded = quote! {
        #ident
        #(#stmts)*
    };

    expanded.into()
}
