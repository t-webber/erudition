use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, LitStr, parse};

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
