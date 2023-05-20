mod trait_item_type;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::borrow::Cow;
use syn::ext::IdentExt;
use syn::parse::{Nothing, Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Attribute, FnArg, Generics, Ident, ImplItem, ImplItemConst,
    ImplItemFn, ImplItemType, ItemImpl, Pat, PatIdent, PatReference, PatTuple, PatType, Receiver,
    Signature, Visibility,
};

use crate::trait_item_type::ImplItemBound;

struct ItemImplWithVisibility {
    attrs: Vec<Attribute>,
    visibility: Visibility,
    impl_item: ItemImpl,
}

impl Parse for ItemImplWithVisibility {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let impl_item = input.parse()?;
        Ok(Self {
            attrs,
            visibility,
            impl_item,
        })
    }
}

/// Declares an extension trait
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate extension_trait;
///
/// #[extension_trait]
/// pub impl DoubleExt for str {
///    fn double(&self) -> String {
///        self.repeat(2)
///    }
/// }
///
/// fn main() {
///     assert_eq!("Hello".double(), "HelloHello");
/// }
/// ```
#[proc_macro_attribute]
pub fn extension_trait(args: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(args as Nothing);
    let ItemImplWithVisibility {
        attrs,
        visibility,
        impl_item,
    } = parse_macro_input!(input as ItemImplWithVisibility);
    let ItemImpl {
        impl_token,
        unsafety,
        trait_,
        items,
        ..
    } = &impl_item;
    let trait_items = items.iter().map(|item| {

        match item {
            ImplItem::Const(ImplItemConst {
                attrs,
                vis: _,
                defaultness: None,
                const_token,
                ident,
                generics: Generics {
                    lt_token: None,
                    params: _,
                    gt_token: None,
                    where_clause: None,
                },
                colon_token,
                ty,
                eq_token: _,
                expr: _,
                semi_token,
            }) => {
                quote! { #(#attrs)* #const_token #ident #colon_token #ty #semi_token }
            },
            ImplItem::Fn(
               impl_item_fn @ImplItemFn {
                attrs,
                vis: _,
                defaultness: None,
                sig: Signature {
                    constness,
                    asyncness,
                    unsafety,
                    abi,
                    fn_token,
                    ident,
                    generics,
                    paren_token: _,
                    inputs,
                    variadic,
                    output
                },
                block: _,
            }) => {
                let inputs = inputs.into_iter().map(|arg| {
                    let span = arg.span();
                    match arg {
                        FnArg::Typed(PatType { attrs, pat, colon_token, ty }) => {
                            let ident = extract_ident(pat).unwrap_or_else(|| Cow::Owned(Ident::new("_", span)));
                            quote! { #(#attrs)* #ident #colon_token #ty }
                        },
                        FnArg::Receiver(Receiver {
                            attrs,
                            reference: None,
                            mutability: _,
                            self_token,
                            colon_token: Some(colon),
                            ty,
                        }) => quote! { #(#attrs)* #self_token #colon #ty },
                        FnArg::Receiver(receiver) => receiver.into_token_stream(),
                    }
                });
                let new_attrs:Vec<Attribute> = attrs.iter().filter(|a| !a.to_token_stream().to_string().contains("intrait")).cloned().collect();
                let is_in_trait =new_attrs.len()<attrs.len();
                if is_in_trait{
                    let mut impl_item_fn = impl_item_fn.clone();
                    impl_item_fn.attrs = new_attrs;
                    quote!{
                        #impl_item_fn
                    }
                }else{
                    let where_clause = &generics.where_clause;
                    quote! {
                        #(#new_attrs)*
                        #constness #asyncness #unsafety #abi #fn_token #ident #generics (#(#inputs,)* #variadic) #output #where_clause;
                    }
                }
            },
            ImplItem::Type(ImplItemType {
                attrs,
                type_token,
                ident,
                generics,
                semi_token,
                ..
            }) => {
                quote! { #(#attrs)* #type_token #ident #generics #semi_token }
            },
                ImplItem::Verbatim(ts) => {
                    let ImplItemBound {
                        attrs,
                        type_token,
                        ident,
                        generics,
                        semi_token,
                        colon_token,
                        bounds,
                        ..
                    }= parse_quote!(#ts);
                    quote! { #(#attrs)* #type_token #ident #generics #colon_token #bounds #semi_token}
                },
            _ => syn::Error::new(item.span(), format!("unsupported item type {}",item.to_token_stream())).to_compile_error(),
        }
    });
    let impl_items = items.iter().filter_map(|item| match item {
        ImplItem::Fn(
            impl_item_fn @ ImplItemFn {
                attrs,
                vis: _,
                defaultness: None,
                sig:
                    Signature {
                        constness,
                        asyncness,
                        unsafety,
                        abi,
                        fn_token,
                        ident,
                        generics,
                        paren_token: _,
                        inputs,
                        variadic,
                        output,
                    },
                block: _,
            },
        ) => {
            let new_attrs: Vec<Attribute> = attrs
                .iter()
                .filter(|a| !a.to_token_stream().to_string().contains("intrait"))
                .cloned()
                .collect();
            let is_in_trait = new_attrs.len() < attrs.len();
            if !is_in_trait {
                Some(item.clone())
            } else {
                None
            }
        }
        ImplItem::Verbatim(ts) => {
            let ImplItemBound {
                attrs,
                type_token,
                ident,
                generics,
                semi_token,
                eq_token,
                expr,
                ..
            } = parse_quote!(#ts);
            let new_item: ImplItem =
                parse_quote! { #(#attrs)* #type_token #ident #generics #eq_token #expr #semi_token};
            Some(new_item)
        }
        _v => Some(_v.clone()),
    });
    let mut impl_item = impl_item.clone();
    impl_item.items = impl_items.collect();

    if let Some((None, path, _)) = trait_ {
        (quote! {
            #(#attrs)*
            #visibility #unsafety trait #path {
                #(#trait_items)*
            }
            #impl_item
        })
        .into()
    } else {
        syn::Error::new(impl_token.span(), "extension trait name was not provided")
            .to_compile_error()
            .into()
    }
}

fn extract_ident(pat: &Pat) -> Option<Cow<'_, Ident>> {
    match pat {
        Pat::Reference(PatReference { pat, .. }) => extract_ident(pat),
        Pat::Ident(PatIdent { ident, .. }) => Some(Cow::Borrowed(ident)),
        Pat::Tuple(PatTuple { elems, .. }) => {
            if elems.len() <= 1 {
                extract_ident(elems.into_iter().next()?)
            } else {
                let span = elems.span();
                let elems = elems
                    .into_iter()
                    .map(extract_ident)
                    .map(|o| o.map(|ident| ident.unraw().to_string()))
                    .collect::<Option<Vec<String>>>()?;
                let joined = elems.join("_");
                Some(Cow::Owned(Ident::new(&joined, span)))
            }
        }
        _ => None,
    }
}
