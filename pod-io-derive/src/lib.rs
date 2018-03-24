#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::Term;

#[proc_macro_derive(Decode, attributes(LE, BE, Arg, Parameter))]
pub fn decode_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_decode(ast);
    gen.into()
}

fn is_outer_attr(attr: &&syn::Attribute) -> bool {
    attr.style == syn::AttrStyle::Outer && !attr.is_sugared_doc //&& !attr.path.segments.is_empty()
}

fn is_arg_attr(attr: &&syn::Attribute) -> bool {
    if let Some(seg) = attr.path.segments.first() {
        seg.value().ident == syn::Ident::from("Arg")
    } else {
        false
    }
}

fn is_parameter_attr(attr: &&syn::Attribute) -> bool {
    if let Some(seg) = attr.path.segments.first() {
        seg.value().ident == syn::Ident::from("Parameter")
    } else {
        false
    }
}

fn is_byte_order_attr(attr: &&syn::Attribute) -> bool {
    if let Some(seg) = attr.path.segments.first() {
        seg.value().ident == syn::Ident::from("BE") ||
        seg.value().ident == syn::Ident::from("LE")
    } else {
        false
    }
}

fn attr_to_term(attr: &syn::Attribute) -> Term {
    let meta = attr.interpret_meta().expect("Can't parse your custom attribute");
    if let syn::Meta::NameValue(meta_namevalue) = meta {
        if let syn::Lit::Str(s) = meta_namevalue.lit {
            //return syn::Ident::from(s.value())
            return Term::intern(&s.value())
        }
    }
    panic!("This attribute is not the form of #[Parameter = \"YourType\"] or #[Arg = \"YourArg\"]")
}

fn attr_to_byte_order(attr: &syn::Attribute) -> syn::Ident {
    let meta = attr.interpret_meta().expect("Can't parse your custom attribute");
    if let syn::Meta::Word(ident) = meta {
        ident
    } else {
        panic!("This attribute is neither #[BE] nor #[LE]")
    }
}

fn impl_decode(ast: syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let argty = ast.attrs.iter().filter(is_outer_attr)
                                .filter(is_parameter_attr).next()
                                .map(attr_to_term)
                                .unwrap_or(Term::intern("Nil"));
    if let syn::Data::Struct(data) = ast.data {
        match data.fields {
            syn::Fields::Named(fields) => {
                let ident: Vec<Option<syn::Ident> > = fields.named.iter().map(|f| f.ident).collect();
                let ident_clone = ident.clone();
                let ty = fields.named.iter().map(|f| f.ty.clone());
                let byte_order = fields.named.iter().map(|f|
                    f.attrs.iter().filter(is_outer_attr)
                                .filter(is_byte_order_attr)
                                .next().map(attr_to_byte_order)
                                .unwrap_or(syn::Ident::from("LE"))
                );
                let arg = fields.named.iter().map(|f|
                    f.attrs.iter().filter(is_outer_attr)
                                .filter(is_arg_attr).next()
                                .map(attr_to_term)
                                .unwrap_or(Term::intern("Nil"))
                );
                quote! {
                    #[allow(unused_variables)]
                    impl<'a, R: ::std::io::Read> Decode<R, #argty> for #name {
                        fn decode<T: ByteOrder>(r: &mut R, p: #argty) -> ::std::io::Result<#name> {
                            #( let #ident = <#ty>::decode::<#byte_order>(r, #arg)?; )*
                            Ok(#name {
                                #(#ident_clone),*
                            })
                        }
                    }
                }
            },
            syn::Fields::Unnamed(_fields) => unimplemented!(),
            syn::Fields::Unit => panic!("can't derive Decode for unit struct"),
        }
    } else {
        panic!("derive Decode for enum/union is not ready");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
