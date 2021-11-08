use quote::quote;
use syn;
use proc_macro::TokenStream;

#[proc_macro_derive(EnumVariantCount)]
pub fn derive_num_variants(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let len = match ast.data {
        syn::Data::Enum(enumeration) => enumeration.variants.len(),
        _ => panic!("enum_variant_count on non enum"),
    };
    let gen = quote! {
        impl #name { 
            fn num_variants() -> usize {
                return #len;
            }
        }
    };
    return gen.into();
}
