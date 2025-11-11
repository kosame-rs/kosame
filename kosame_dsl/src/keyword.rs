macro_rules! custom_keyword {
    ($kw:ident) => {
        ::syn::custom_keyword!($kw);

        impl $kw {
            #[allow(unused)]
            pub fn parse_autocomplete(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                let result = input.parse::<Self>();
                match result {
                    Ok(result) => Ok(Self { span: result.span }),
                    Err(error) => {
                        let span = input.cursor().span();
                        let ident = input.parse::<::syn::Ident>().ok();

                        ::proc_macro_error::dummy::set_dummy(::quote::quote! {
                            { use ::kosame::keyword::$kw::#ident; () }
                        });
                        ::proc_macro_error::abort!(span, error.to_string());
                    }
                }
            }
        }
    };
}

macro_rules! keyword_group {
    ($group:ident { $($kw:ident),* }) => {
        #[allow(non_camel_case_types)]
        pub struct $group {}
        impl $group {
            #[allow(unused)]
            pub fn error(input: ::syn::parse::ParseStream) -> ! {
                let lookahead = input.lookahead1();
                $(lookahead.peek($kw);)*
                let error = lookahead.error();

                let span = input.cursor().span();
                let ident = input.parse::<::syn::Ident>().ok();

                ::proc_macro_error::dummy::set_dummy(::quote::quote! {
                    { use ::kosame::keyword::$group::#ident; () }
                });
                ::proc_macro_error::abort!(span, error.to_string());
            }
        }
    };
}

custom_keyword!(and);
custom_keyword!(asc);
custom_keyword!(by);
custom_keyword!(cast);
custom_keyword!(create);
custom_keyword!(cross);
custom_keyword!(default);
custom_keyword!(delete);
custom_keyword!(desc);
custom_keyword!(distinct);
custom_keyword!(driver);
custom_keyword!(first);
custom_keyword!(from);
custom_keyword!(full);
custom_keyword!(group);
custom_keyword!(having);
custom_keyword!(inner);
custom_keyword!(insert);
custom_keyword!(into);
custom_keyword!(is);
custom_keyword!(join);
custom_keyword!(key);
custom_keyword!(kosame);
custom_keyword!(last);
custom_keyword!(lateral);
custom_keyword!(left);
custom_keyword!(limit);
custom_keyword!(natural);
custom_keyword!(not);
custom_keyword!(null);
custom_keyword!(nulls);
custom_keyword!(offset);
custom_keyword!(on);
custom_keyword!(or);
custom_keyword!(order);
custom_keyword!(__pass);
custom_keyword!(primary);
custom_keyword!(references);
custom_keyword!(rename);
custom_keyword!(returning);
custom_keyword!(right);
custom_keyword!(select);
custom_keyword!(set);
custom_keyword!(__table);
custom_keyword!(table);
custom_keyword!(ty);
custom_keyword!(update);
custom_keyword!(using);
custom_keyword!(values);
custom_keyword!(with);

keyword_group!(group_attribute { driver, rename, ty });
keyword_group!(group_column_constraint {
    not,
    default,
    primary,
    references
});
keyword_group!(group_command {
    select,
    insert,
    update,
    delete
});
keyword_group!(group_join {
    left,
    right,
    inner,
    full,
    natural,
    cross
});
keyword_group!(group_order_by_dir { asc, desc });
keyword_group!(group_order_by_nulls { first, last });
