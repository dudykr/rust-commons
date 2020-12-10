use pmutil::q;
use pmutil::IdentExt;
use pmutil::Quote;
use pmutil::SpanExt;
use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Attribute;
use syn::Block;
use syn::ExprStruct;
use syn::Field;
use syn::FieldValue;
use syn::Fields;
use syn::FieldsNamed;
use syn::Generics;
use syn::Item;
use syn::ItemFn;
use syn::ItemStruct;
use syn::ReturnType;
use syn::Signature;
use syn::Stmt;
use syn::Token;
use syn::Visibility;

pub fn expand(injector: ItemFn) -> Vec<Item> {
    let injector_struct_ident = injector
        .sig
        .ident
        .new_ident_with(|name| format!("{}_struct", name));

    let mut items = vec![];
    let mut fields_for_struct = Punctuated::<Field, Token![,]>::default();
    let mut injector_body: Vec<Stmt> = vec![];
    let mut fields_for_init = Punctuated::<FieldValue, Token![,]>::default();

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

                fields_for_struct.push(Field {
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

    for stmt in &injector.block.stmts {
        match stmt {
            Stmt::Item(Item::Fn(provider)) => {
                //
                let var_name = provider
                    .sig
                    .ident
                    .new_ident_with(|mtd_name| format!("var_{}", mtd_name));
                injector_body.push(
                    q!(
                        Vars {
                            var_name: &var_name,
                            mtd_name: &provider.sig.ident
                        },
                        {
                            let var_name = mtd_name();
                        }
                    )
                    .parse(),
                );

                fields_for_init.push(
                    q!(
                        Vars {
                            var_name,
                            mtd_name: &provider.sig.ident
                        },
                        { mtd_name: var_name }
                    )
                    .parse(),
                );
            }
            _ => todo!("Currently the function with #[injector] should have only function items"),
        }
    }

    for field in &fields_for_struct {
        // Implement rdi::Value<T>
        items.push(
            q!(
                Vars {
                    name: &field.ident,
                    ty: &field.ty,
                    injector_struct_ident: &injector_struct_ident
                },
                {
                    impl rdi::Value<ty> for injector_struct_ident {
                        fn value(&self) -> ty {
                            self.name
                        }
                    }
                }
            )
            .parse(),
        );
    }

    {
        // Create a hidden struct which stores all the values.
        let injector_struct = ItemStruct {
            attrs: take_attrs(q!({
                #[allow(non_camel_case_types)]
                struct Useless;
            })),
            vis: injector.vis.clone(),
            struct_token: Span::call_site().as_token(),
            ident: injector_struct_ident.clone(),
            // TODO: Support generics
            generics: Generics::default(),
            fields: Fields::Named(FieldsNamed {
                brace_token: Span::call_site().as_token(),
                named: fields_for_struct,
            }),
            semi_token: None,
        };
        items.push(Item::Struct(injector_struct));
    }

    injector_body.extend(injector.block.stmts);

    {
        // Create a function which returns injector
        injector_body.push(
            q!(
                Vars {
                    injector_struct_ident: &injector_struct_ident,
                    fields_for_init,
                },
                {
                    return injector_struct_ident { fields_for_init };
                }
            )
            .parse(),
        );

        let creator = ItemFn {
            attrs: Default::default(),
            vis: injector.vis,
            sig: Signature {
                output: ReturnType::Type(
                    Span::call_site().as_token(),
                    q!(
                        Vars {
                            injector_struct_ident: &injector_struct_ident
                        },
                        { injector_struct_ident }
                    )
                    .parse(),
                ),
                ..injector.sig.clone()
            },
            block: Box::new(Block {
                brace_token: injector.block.brace_token,
                stmts: injector_body,
            }),
        };

        items.push(creator.into())
    }

    {
        // Make `inject`.
        items.push(
            q!(
                Vars {
                    injector_struct_ident: &injector_struct_ident
                },
                {
                    impl injector_struct_ident {
                        pub fn inject<'a, T>(&self, t: T) -> T::Output
                        where
                            Self: rdi::Provider<T::Injected>,
                            T: rdi::Injectable<'a>,
                        {
                            let injected = rdi::Provider::provide(self);
                            t.inject(injected)
                        }
                    }
                }
            )
            .parse(),
        );
    }

    items
}

fn take_attrs(q: Quote) -> Vec<Attribute> {
    q.parse::<ItemStruct>().attrs
}
