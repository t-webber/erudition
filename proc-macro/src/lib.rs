use std::sync::Mutex;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Ident, ItemFn, LitStr, Pat, parse};

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

    let args = match function
        .sig
        .inputs
        .iter()
        .filter_map(|input| {
            if let syn::FnArg::Typed(pat_type) = input
                && let Pat::Ident(pat_ident) = *pat_type.pat.clone()
            {
                let ident = pat_ident.ident;
                let string = ident.to_string();
                if string == "state" {
                    None
                } else {
                    Some(Ok(quote!(
                        log.push_str(&format!(" [{}: {:?}]",
                            #string,
                            &#ident
                        ));
                    )))
                }
            } else {
                Some(Err(error(
                    quote!(#input).into(),
                    "invalid function argument",
                )))
            }
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        Err(error) => return error,
    };

    ROUTES.with(|v| v.lock().unwrap().push(name.to_string()));

    let expanded = quote! {
        #[allow(clippy::literal_string_with_formatting_args)]
        #[actix_web::#request(#path)]
        #sig {
            let mut log = String::new();
            #(#args)*
            let res = HttpResponse::from({ #block });
            state.log(&format!("{} ({}){}", stringify!(#request), res.status().as_u16(), &log));
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
