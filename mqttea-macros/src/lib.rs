pub(crate) mod length;
pub(crate) mod impl_u8;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataEnum, DeriveInput};



#[proc_macro_derive(Length, attributes(bytes))]
pub fn derive_length(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as DeriveInput);
    let struct_name = _input.ident;

    let syn::Data::Struct(data_struct) = &_input.data else { return TokenStream::new() };
    let syn::Fields::Named(fields) = &data_struct.fields else { return TokenStream::new() };

    let mut field_lens: Vec<proc_macro2::TokenStream> = vec![];

    for field in &fields.named {
        let attrs = &field.attrs;
        let has_bytes_attr = attrs.iter().find_map(|attr| {
            let syn::Attribute{meta, ..} = attr;
            let syn::Meta::List(syn::MetaList{path, tokens, ..}) = meta else {return None};
            let syn::Path{segments, ..} = path;
            if segments.first().is_some_and(|s| s.ident.to_string() == "bytes") { return Some(tokens) };
            return None;
        });

        if has_bytes_attr.is_some_and(|token| token.to_string().as_str() == "ignore") { continue }
        let add_id_to_length = !(has_bytes_attr.is_some_and(|token| token.to_string().as_str() == "no_id")); // check if the field has a no_id(no identifier attribute)

        field_lens.push(length::calculate(&field, add_id_to_length));
    }

    let output = quote! {
        impl #struct_name {
            fn len(&self) -> usize {
                let mut size = 0;
                #( #field_lens )*
                return size;
            }

            fn variable_len(int: usize) -> usize {
                if int >= 2_097_152 { return 4 }
                if int >= 16_384 { return 3 }
                if int >= 128 { return 2 }
                return 1
            }
        }
    };


    TokenStream::from(output)
}



#[proc_macro_derive(FromU8)]
pub fn derive_u8(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as DeriveInput);
    let struct_name = _input.ident;

    let syn::Data::Enum(DataEnum { variants, .. }) = _input.data else { 
        return syn::Error::new(struct_name.span(), "FromU8 can only be derived for enums").to_compile_error().into()
        // return TokenStream::new()
     };
    
    let variant_pair = variants.iter().map(|v| {
        match &v.discriminant {
            Some((_, discriminant)) => {
                let discriminant_value = quote! { #discriminant };
                Ok((&v.ident, discriminant_value))
            }
            _ => Err(syn::Error::new(v.ident.span(), "Please provide a valid discriminant for this variant")),
        }
    }).collect::<Vec<Result<(&syn::Ident, proc_macro2::TokenStream), syn::Error>>>();

    let (variant, discriminant): (Vec<_>, Vec<_>) = variant_pair.into_iter().filter_map(|d| d.ok()).unzip();
    
    let try_from_u8 = quote! {
        impl TryFrom<u8> for #struct_name {
            type Error = String;
            
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    #( #discriminant => Ok(#struct_name::#variant), )*
                    v => Err((format!("{}", v)))
                }
            }
        }
    };

    
    let output = quote! {
        impl From<#struct_name> for u8 {
            fn from(n: #struct_name) -> Self {
                n as u8
            }
        }

        #try_from_u8
    };

    TokenStream::from(output)
}