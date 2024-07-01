extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ImplNew)]
pub fn impl_new_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            fields
        } else {
            panic!("ImplNew can only be used with named fields");
        }
    } else {
        panic!("ImplNew can only be used with structs");
    };

    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();

    let expanded = quote! {
        impl #name {
            pub fn new(row: Vec<VOTableValue>) -> Self {
                let mut iter = row.into_iter();
                #name {
                    #(
                        #field_names: match iter.next() {
                            Some(VOTableValue::String(val)) => Some(val),
                            Some(VOTableValue::Double(val)) => Some(val),
                            Some(VOTableValue::Int(val)) => Some(val as i32),
                            Some(VOTableValue::CharASCII(val)) => Some(val.to_string()),
                            Some(VOTableValue::CharUnicode(val)) => Some(val.to_string()),
                            Some(VOTableValue::Null) | None => None,
                            _ => None,
                        },
                    )*
                }
            }
        }
    };

    println!("Expanded: {}", expanded);
    TokenStream::from(expanded)
}
