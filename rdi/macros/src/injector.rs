use pmutil::IdentExt;
use pmutil::SpanExt;
use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Block;
use syn::Expr;
use syn::ExprReturn;
use syn::Field;
use syn::Fields;
use syn::FieldsNamed;
use syn::Generics;
use syn::Item;
use syn::ItemFn;
use syn::ItemStruct;
use syn::ReturnType;
use syn::Stmt;
use syn::Token;
use syn::Visibility;

pub fn expand(injector: ItemFn) -> Vec<Item> {
    let injector_struct_ident = injector
        .sig
        .ident
        .new_ident_with(|name| format!("{}_struct", name));

    let mut items = vec![];
    let mut fields = Punctuated::<Field, Token![,]>::default();

    for stmt in &injector.block.stmts {
        match stmt {
            Stmt::Item(Item::Fn(provider)) => {
                //
                let ret_ty = &provider.sig.output;
                let (ret_arrow_token, ret_ty) = match ret_ty {
                    ReturnType::Default => {
                        panic!("Provider method should have return type");
                    }
                    ReturnType::Type(tok, ty) => (tok, ty),
                };

                fields.push(Field {
                    attrs: vec![],
                    vis: Visibility::Inherited,
                    ident: Some(provider.sig.ident.clone()),
                    colon_token: Some(ret_arrow_token.span().as_token()),
                    ty: *ret_ty.clone(),
                });
            }
            _ => todo!("Currently the function with #[injector] should have only function items"),
        }
    }

    {
        // Create a hidden struct which stores all the values.
        let injector_struct = ItemStruct {
            attrs: vec![],
            vis: Visibility::Inherited,
            struct_token: Span::call_site().as_token(),
            ident: injector_struct_ident,
            // TODO: Support generics
            generics: Generics::default(),
            fields: Fields::Named(FieldsNamed {
                brace_token: Span::call_site().as_token(),
                named: fields,
            }),
            semi_token: None,
        };
        items.push(injector_struct.into());
    }

    {
        // Create a function which returns injector
        let creator = ItemFn {
            attrs: Default::default(),
            vis: injector.vis,
            sig: injector.sig.clone(),
            block: Box::new(Block {
                brace_token: injector.block.brace_token,
                stmts: vec![Stmt::Expr(Expr::Return(ExprReturn {}))],
            }),
        };

        items.push(creator.into())
    }

    items
}
