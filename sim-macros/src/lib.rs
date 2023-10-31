
use proc_macro::TokenStream;
use syn::parse;
use syn::*;
use syn::punctuated::*;
use syn::token::*;
use quote::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldKind {
    Input, Output, Clocked, Memory, None
}
struct FieldInfo {
    name: syn::Ident,
    ty: syn::Type,
    inner_ty: syn::Type,
    kind: FieldKind,
}

fn extract_concrete_type(name: &syn::Ident, ty: &syn::Type) -> syn::Type {
    let path = match ty {
        syn::Type::Path(ref typepath) if typepath.qself.is_none() => {
            &typepath.path
        },
        _ => panic!("Couldn't get path for type"),
    };

    let x = match path.segments.iter().filter(|t| {
        t.ident == "Signal" || t.ident == "Register" || t.ident == "VecSignal"
    }).last() {
        Some(s) => s,
        _ => panic!("Field '{}' must be Signal<T> or Register<T>", name),
    };

    match x.arguments {
        syn::PathArguments::AngleBracketed(ref params) => {
            match params.args.first() {
                Some(syn::GenericArgument::Type(ref ty)) => ty.clone(),
                _ => panic!("Field '{}' has invalid type paramers for '{}'", 
                            name, x.ident),
            }
        },
        _ => panic!("Error parsing field '{}' with type '{}'", name, x.ident),
    }
}

#[proc_macro_derive(Module, attributes(input, output, clocked, memory))]
pub fn derive_module(tokens: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(tokens as syn::DeriveInput);

    let s: DataStruct = match ast.data {
        syn::Data::Struct(ref s) => s.clone(),
        _ => panic!("Can only derive Module for structs"),
    };
    let struct_name = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let fields: Punctuated<Field, Comma> = match s.fields {
        syn::Fields::Named(FieldsNamed { ref named, ..}) => named.clone(),
        _ => panic!("Can only derive Module for structs with named fields"),
    };

    // Capture information from all of the fields
    let mut field_info = Vec::new();
    for field in fields.iter() {
        let name = field.ident.clone().unwrap();
        let ty   = field.ty.clone();
        let inner_ty = extract_concrete_type(&name, &ty);

        let kind = {
            let mut res = FieldKind::None;
            for attr in field.attrs.iter() {
                let attr_name = attr.path.get_ident().unwrap().to_string();
                match attr_name.as_str() {
                    "input" => { res = FieldKind::Input; break; },
                    "output" => { res = FieldKind::Output; break; },
                    "clocked" => { res = FieldKind::Clocked; break; },
                    "memory" => { res = FieldKind::Memory; break; },
                    _ => {},
                }
            }
            res
        };
        field_info.push(FieldInfo { 
            name: field.ident.clone().unwrap().clone(),
            ty: ty,
            inner_ty: inner_ty,
            kind, 
        });
    }

    let mut output = TokenStream::new();

    // Inputs
    let names: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Input)
        .map(|f| f.name.clone())
        .collect();
    let types: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Input)
        .map(|f| f.ty.clone())
        .collect();
    let inner_types: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Input)
        .map(|f| f.inner_ty.clone())
        .collect();


    // Automatically implement public methods for driving input wires
    let drive_fn_names: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Input)
        .map(|f| format_ident!("drive_{}", f.name))
        .collect();
    output.extend(Into::<TokenStream>::into(quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(
                pub fn #drive_fn_names(&mut self, value: #inner_types) {  
                    self.#names.drive(value);
                }
            )*
        }
    }));

    // Outputs
    let names: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Output)
        .map(|f| f.name.clone())
        .collect();
    let types: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Output)
        .map(|f| f.ty.clone())
        .collect();
    let inner_types: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Output)
        .map(|f| f.inner_ty.clone())
        .collect();

    // Automatically implement public methods for sampling output wires 
    let sample_fn_names: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Output)
        .map(|f| format_ident!("sample_{}", f.name))
        .collect();
    output.extend(Into::<TokenStream>::into(quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(
                pub fn #sample_fn_names(&self) -> #inner_types {  
                    self.#names.sample()
                }
            )*
        }
    }));


    //let output_type_name = format_ident!("{}_Outputs", struct_name);
    //output.extend(Into::<TokenStream>::into(quote! {
    //    pub struct #output_type_name {
    //        #(#names: #types),*
    //    }
    //}));
    //output.extend(Into::<TokenStream>::into(quote! {
    //    impl #struct_name {
    //        pub fn sample(&self) -> #output_type_name {
    //            #output_type_name {
    //                #(#names: self.#names,)*
    //            }
    //        }
    //    }
    //}));

    // Registers
    let names: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Clocked)
        .map(|f| f.name.clone())
        .collect();
    let types: Vec<_> = field_info.iter()
        .filter(|f| f.kind == FieldKind::Clocked)
        .map(|f| f.ty.clone())
        .collect();

    // Automatically implement a function which propagates a simulated clock
    // edge to all members that have been marked as 'clocked'.
    output.extend(Into::<TokenStream>::into(quote! {
        impl #impl_generics Clocked for #struct_name #ty_generics #where_clause {
            fn sim_clock_edge(&mut self) {
                #(self.#names.sim_clock_edge();)*

            }
        }
    }));

    output.into()
}

//#[proc_macro_derive(Bundle, attributes(input, output))]
//pub fn derive_bundle(tokens: TokenStream) -> TokenStream {
//    let ast = parse_macro_input!(tokens as syn::DeriveInput);
//
//    let s: DataStruct = match ast.data {
//        syn::Data::Struct(ref s) => s.clone(),
//        _ => panic!("Can only derive Bundle for structs"),
//    };
//    let struct_name = ast.ident;
//
//    let fields: Punctuated<Field, Comma> = match s.fields {
//        syn::Fields::Named(FieldsNamed { ref named, ..}) => named.clone(),
//        _ => panic!("Can only derive Bundle for structs with named fields"),
//    };
//
//    let mut field_info = Vec::new();
//    for field in fields.iter() {
//        let name = field.ident.clone().unwrap();
//        let ty   = field.ty.clone();
//        let inner_ty = extract_concrete_type(&name, &ty);
//
//        let kind = {
//            let mut res = FieldKind::None;
//            for attr in field.attrs.iter() {
//                let attr_name = attr.path.get_ident().unwrap().to_string();
//                match attr_name.as_str() {
//                    "input" => { res = FieldKind::Input; break; },
//                    "output" => { res = FieldKind::Output; break; },
//                    "clocked" => { res = FieldKind::Clocked; break; },
//                    "memory" => { res = FieldKind::Memory; break; },
//                    _ => {},
//                }
//            }
//            res
//        };
//        field_info.push(FieldInfo { 
//            name: field.ident.clone().unwrap().clone(),
//            ty: ty,
//            inner_ty: inner_ty,
//            kind, 
//        });
//    }
//    let mut output = TokenStream::new();
//    output.into()
//}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
