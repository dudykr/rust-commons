use pmutil::q;
use pmutil::SpanExt;
use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::Attribute;
use syn::FnArg;
use syn::Item;
use syn::ItemFn;
use syn::Pat;
use syn::PatTuple;
use syn::ReturnType;
use syn::Token;
use syn::Type;
use syn::TypeTuple;

pub fn expand(f: ItemFn) -> Vec<Item> {
    let mut result: Vec<Item> = vec![];
    let name = &f.sig.ident;

    result.push(
        q!(Vars { name }, {
            #[allow(non_camel_case_types)]
            struct name;
        })
        .parse(),
    );

    {
        let mut injected_pat_elems = Punctuated::<_, Token![,]>::default();
        let mut injected_type_elems = Punctuated::<_, Token![,]>::default();
        let mut extra_pat_elems = Punctuated::<_, Token![,]>::default();
        let mut extra_type_elems = Punctuated::<_, Token![,]>::default();

        for input in f.sig.inputs {
            match input {
                FnArg::Receiver(_) => continue,
                FnArg::Typed(pat_ty) => {
                    // If it's injected, we
                    if has_inject(&pat_ty.attrs) {
                        injected_pat_elems.push(*pat_ty.pat);
                        injected_type_elems.push(*pat_ty.ty);
                    } else {
                        extra_pat_elems.push(*pat_ty.pat);
                        extra_type_elems.push(*pat_ty.ty);
                    }
                }
            }
        }

        if !injected_type_elems.trailing_punct() {
            injected_type_elems.push_punct(Span::call_site().as_token());
        }
        if !injected_pat_elems.trailing_punct() {
            injected_pat_elems.push_punct(Span::call_site().as_token());
        }

        let injected_pat = Pat::Tuple(PatTuple {
            attrs: Default::default(),
            paren_token: f.sig.paren_token,
            elems: injected_pat_elems,
        });

        let injected_type = Type::Tuple(TypeTuple {
            paren_token: f.sig.paren_token,
            elems: injected_type_elems,
        });

        let extra_input_pat = Pat::Tuple(PatTuple {
            attrs: Default::default(),
            paren_token: f.sig.paren_token,
            elems: extra_pat_elems,
        });
        let is_extra_empty = extra_type_elems.is_empty();
        let extra_type = Type::Tuple(TypeTuple {
            paren_token: f.sig.paren_token,
            elems: extra_type_elems,
        });

        let body = f.block;
        let ret_ty = match f.sig.output {
            ReturnType::Default => q!({ () }),
            ReturnType::Type(_, ty) => q!(Vars { ty }, { ty }),
        };
        //
        let (extra_input_pat, extra_type) = if !is_extra_empty {
            (
                q!(Vars { extra_input_pat }, { extra_input_pat }),
                q!(Vars { extra_type }, { extra_type }),
            )
        } else {
            (q!({}), q!({}))
        };

        result.push(
            q!(
                Vars {
                    extra_type,
                    extra_input_pat,
                    ret_ty,
                    injected_pat,
                    injected_type,
                    body,
                },
                {
                    impl<'a> rdi::Injectable<'a> for handler {
                        type Output = Box<dyn Fn(extra_type) -> ret_ty>;
                        type Injected = injected_type;

                        fn inject(self, injected_pat: Self::Injected) -> Self::Output {
                            Box::new(move |extra_input_pat| body)
                        }
                    }
                }
            )
            .parse(),
        )
    }

    result
}

pub(crate) fn has_inject(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if attr.path.segments.last().unwrap().ident.to_string() == "inject" {
            return true;
        }
    }

    false
}
