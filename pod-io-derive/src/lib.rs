extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Decode, attributes(LE, BE))]
pub fn decode_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = impl_decode(ast);
    gen.into()
}

fn impl_decode(ast: syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    if let syn::Data::Struct(data) = ast.data {
        match data.fields {
            syn::Fields::Named(fields) => {
                let ident: Vec<Option<syn::Ident> > = fields.named.iter().map(|f| f.ident).collect();
                let ident_clone = ident.clone();
                let ty = fields.named.iter().map(|f| f.ty.clone());
                let byte_order = fields.named.iter().map(|f| -> syn::Ident {
                    for attr in f.attrs.iter() {
                        if attr.style == syn::AttrStyle::Outer && !attr.is_sugared_doc && !attr.path.segments.is_empty() {
                            if let Some(seg) = attr.path.segments.first() {
                                return seg.value().ident;
                            }
                        }
                    }
                    syn::Ident::from("LE")
                });
                quote! {
                    impl Decode for #name {
                        type Output = #name;
                        fn decode<T: ByteOrder, R: std::io::Read>(r: &mut R) -> std::io::Result<#name> {
                            #( let #ident = <#ty>::decode::<#byte_order, _>(r)?; )*
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
