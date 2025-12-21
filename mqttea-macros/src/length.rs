use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, PathSegment, Ident, PathArguments, Type, TypePath};



/// Currently only handles:
/// Option<u8|u16|u32|u128|String|bool|Bytes|usize>,
/// Vec<String|Bytes>, Vec<(String|Bytes, String|Bytes)>
/// u8|u16|u32|u128|true|String|Bytes
/// We allocate an extra 
pub(crate) fn calculate(field: &Field, include_id: bool) -> TokenStream {
    let result = quote! { size += 0; };

    let Type::Path(TypePath{path, ..}) = &field.ty else { return result };
    let Some(segment) = path.segments.last() else { return result };

    let type_name= &segment.ident;
    let data_type = (&segment.ident).to_string();
    let Some(field_name) = &field.ident else { return result };

    return match data_type.as_str() {
        "Option" => { // handles only Option<u8|i16|...>
            let PathArguments::AngleBracketed(args) = &segment.arguments else { return result };
            let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() else { return result };
            match inner_ty {
                syn::Type::Path(inner_path) => {
                    let data_type = inner_path.path.segments.last().unwrap();
                    let type_name = &data_type.ident;
                    let call_size = get_size(field_name, type_name, data_type, true, include_id);
                    quote! { #call_size; }
                }
                _ => {result}
            }
        }
        _ => get_size(field_name, type_name, segment, false, include_id)
    };
}


/// include_id: Whether or not the length of the property(property identification number(u8)) should be included in the length of this field
///     this is only true for fields that are prepended with the property_type(property id)
fn get_size(f_name: &Ident, type_name: &Ident, segment: &PathSegment, is_optional: bool, include_id: bool) -> TokenStream {
    let result = quote! { size +=  0; };

    return match type_name.to_string().as_str() {
        "String" | "Bytes" => {
            // Each of these strings is prefixed with a Two Byte Integer length field that gives the number of bytes in a 261 UTF-8 encoded string itself (1.5.4)
            if is_optional {
                return quote! { if let Some(ref value) = self.#f_name { size += value.len() + 2 + usize::from(#include_id); } } // 2 for the length of the string, and 1 for the property identifier that specifies whatever this value represents
            } else  {
                return quote! {size += self.#f_name.len() + 2 + usize::from(#include_id);} // 2 for the length of the string, and 1 for the property identifier that specifies whatever this value represents
            }
        }
        "Vec" => { // we only expect a vector of (String|Bytes, String|Bytes), String, or Bytes, and now Vec<usize>
            // The first string serves as the name, and the second string contains the value.
            let PathArguments::AngleBracketed(args) = &segment.arguments else { return result };
            let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() else { return result };
            match inner_ty {
                syn::Type::Path(inner_path) => { // Vec<String|Bytes>
                    if inner_path.path.segments.last().filter(|ps| ps.ident == "usize").is_some() {
                        return quote! { 
                            size += self.#f_name.iter().map(|u| {
                                Self::variable_len(*u) + 1
                            }).sum::<usize>();
                     };
                    }
                    
                    if inner_path.path.segments.last().filter(|ps| ps.ident == "String" || ps.ident == "Bytes").is_none() { return result };
                    return quote! { size += self.#f_name.iter().map(|s| s.len() + 2).sum::<usize>(); }
                }
                syn::Type::Tuple(tuple) => { // Vec<(String|Bytes, String|Bytes)>
                    if tuple.elems.len() != 2 { return result };
                    let string_or_bytes = tuple.elems.iter().all(|elem| 
                        if let syn::Type::Path(elem) = elem {
                            // Exception: No instance of Vec<String|Bytes> is ever prepended with an id (so id length is never calculated anyway)
                            if elem.path.segments.last().filter(|ps| ps.ident == "String" || ps.ident == "Bytes").is_some() { return true} else {false}
                        } else {
                            // true
                            false
                        });
                    
                    if !string_or_bytes { return result; }

                    if is_optional {
                        return quote! { if let Some(ref value) = self.#f_name { size += value.iter().map(|(k, v)| k.len() + 2 + v.len() + 2 + usize::from(#include_id)).sum::<usize>(); } };
                    } else {
                        return quote! { size += self.#f_name.iter().map(|(k, v)| k.len() + 2 + v.len() + 2 + usize::from(#include_id)).sum::<usize>(); };
                    }
                }
                _ => { 
                    return result }
            }
        }
        // this should be updated, if there's an unincluded type
        "u8" | "u16" | "u32" | "u64" | "u128" | "bool" => {
            if is_optional {
                return quote! { if self.#f_name.is_some() { size += std::mem::size_of::<#type_name>() + usize::from(#include_id) }; }
            } else {
                return quote! { size += std::mem::size_of::<#type_name>() + usize::from(#include_id); }
            }
        }
        "usize" => {
            if is_optional {
                return quote! { if let Some(ref value) = self.#f_name { size += Self::variable_len(*value) + usize::from(#include_id); } }
            } else {
                return quote! { size += Self::variable_len(self.#f_name) + usize::from(#include_id); }
            }
        }
        _ => {result}
    }
}