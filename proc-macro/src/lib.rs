use std::sync::Mutex;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Ident, ItemFn, LitStr, parse};

thread_local! {
    static ROUTES: Mutex<Vec<String>> = Mutex::default();
}

fn error(attr: TokenStream, msg: &str) -> TokenStream {
    Error::new_spanned(proc_macro2::TokenStream::from(attr), msg)
        .to_compile_error()
        .into()
}

fn logged_actix_request(
    request: proc_macro2::TokenStream,
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

    ROUTES.with(|v| v.lock().unwrap().push(name.to_string()));

    let expanded = quote! {
        #[allow(clippy::literal_string_with_formatting_args)]
        #[actix_web::#request(#path)]
        #sig {
            let res = HttpResponse::from({ #block });
            state.log(&format!("POST ({}): {}", res.status(), #path));
            res
        }
    };

    expanded.into()
}

macro_rules! macros {
    ($($request:ident),*) => {
        $(
            #[proc_macro_attribute]
            pub fn $request(arg: TokenStream, body: TokenStream) -> TokenStream {
            logged_actix_request(quote!($request).into(), arg, body)
            }
        )*
    };
}

macros!(get, post, put);

#[proc_macro]
pub fn routes(app: TokenStream) -> TokenStream {
    let Ok(ident): Result<Ident, _> = parse(app.clone()) else {
        return error(app, "argument should be an identifier");
    };

    let stmts: Vec<_> = ROUTES.with(|v| {
        v.lock()
            .unwrap()
            .iter()
            .map(|name| {
                let ident = Ident::new(name, proc_macro2::Span::call_site());
                quote!(#ident)
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
