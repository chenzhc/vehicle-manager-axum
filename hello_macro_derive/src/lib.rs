use proc_macro::TokenStream;
use syn::DeriveInput;
use syn;
use quote::quote;

extern crate proc_macro;

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    impl_hello_macor(&ast)
}

fn impl_hello_macor(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let r_gen = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };
    r_gen.into()
}