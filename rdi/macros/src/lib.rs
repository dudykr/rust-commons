use pmutil::q;

extern crate proc_macro;

mod inject;
mod injector;

/// Make function arguments injectable.
#[proc_macro_attribute]
pub fn inject(
    _: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input =
        syn::parse(fn_item).expect("#[inject] Currently only supports a standalone function");
    let items = self::inject::expand(input);

    let mut q = q!({});
    for item in &items {
        q.push_tokens(&item)
    }

    q.into()
}

/// Creates an injector.
#[proc_macro_attribute]
pub fn injector(
    _: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse(fn_item).expect("#[injector] expects a standalone function");
    let items = self::injector::expand(input);

    let mut q = q!({});
    for item in &items {
        q.push_tokens(&item)
    }

    q.into()
}
