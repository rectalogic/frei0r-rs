#![feature(iterator_try_collect)]

extern crate proc_macro;

use proc_macro::TokenStream;

use syn::*;
use syn::punctuated::*;

use quote::quote;

use std::ffi::CString;

struct FieldInfo {
    ident : Ident,
    ty : Type,
    rename : Option<Expr>,
    explain : Option<Expr>,
}

impl FieldInfo {
    fn new(field : Field) -> Result<Option<Self>> {
        let mut rename = None;
        let mut explain = None;
        for attr in field.attrs {
            if attr.path().is_ident("internal") {
                return Ok(None);
            }
            else if attr.path().is_ident("frei0r") {
                let name_values: Punctuated<MetaNameValue, Token![,]> = attr.parse_args_with(Punctuated::parse_terminated)?;
                for name_value in name_values {
                    let ident = name_value.path.require_ident()?;
                    match ident {
                        ident if ident == "rename" => {
                            rename = match rename {
                                Some(_) => Err(Error::new_spanned(name_value, "attempting to set rename attribute more than once"))?,
                                None => Some(name_value.value),
                            };
                        },
                        ident if ident == "explain" => {
                            explain = match explain {
                                Some(_) => Err(Error::new_spanned(name_value, "attempting to set explain attribute more than once"))?,
                                None => Some(name_value.value),
                            };
                        },
                        _ => Err(Error::new_spanned(name_value, "unknown attribute"))?,
                    }
                }
            }
        }
        Ok(Some(Self {
            ident : field.ident.unwrap(),
            ty : field.ty,
            rename,
            explain,
        }))
    }

    fn param_name(&self) -> Expr {
        self.rename.clone().unwrap_or_else(|| {
            let ident = self.ident.to_string();
            let ident = CString::new(ident).unwrap();
            let ident = proc_macro2::Literal::c_string(&ident);
            parse_quote! { #ident }
        })
    }

    fn param_explain(&self) -> Expr {
        self.explain.clone().unwrap_or_else(|| parse_quote! { c"" })
    }
}

struct DeriveInputInfo {
    ident : Ident,
    generics : Generics,
    fields : Vec<FieldInfo>,
}

impl DeriveInputInfo {
    fn new(derive_input : DeriveInput) -> Result<Self> {
        match derive_input {
            DeriveInput { ident, generics, data : Data::Struct(DataStruct { fields : Fields::Named(fields), .. }), .. } => Ok(Self {
                ident,
                generics,
                fields : fields.named.into_iter().flat_map(|f| FieldInfo::new(f).transpose()).try_collect()?,
            }),
            _ => Err(Error::new_spanned(derive_input,  "Derive macro PluginBase is only supported on struct with named fields."))
        }
    }
}

/// Derive macro used in the implementation of [PluginBase](../frei0r_rs/trait.PluginBase.html) trait.
#[proc_macro_derive(PluginBase, attributes(frei0r, internal))]
pub fn derive_plugin_base(input : TokenStream) -> TokenStream {
    DeriveInputInfo::new(parse_macro_input!(input as DeriveInput))
        .map(|info| {
            let generics = info.generics;
            let ident = &info.ident;

            let param_count = info.fields.len();
            let param_indices = (0..param_count).collect::<Vec<_>>();

            let param_idents = info.fields.iter().map(|field| field.ident.clone()).collect::<Vec<_>>();
            let param_tys    = info.fields.iter().map(|field| field.ty   .clone()).collect::<Vec<_>>();

            let param_names    = info.fields.iter().map(|field| field.param_name())   .collect::<Vec<_>>();
            let param_explains = info.fields.iter().map(|field| field.param_explain()).collect::<Vec<_>>();

            let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
            quote! {
                unsafe impl #impl_generics ::frei0r_rs::PluginBase for #ident #ty_generics #where_clause {
                    fn param_count() -> usize {
                        #param_count
                    }

                    fn param_info(index : usize) -> ::frei0r_rs::ParamInfo {
                        match index {
                            #(#param_indices => ::frei0r_rs::ParamInfo {
                                name : #param_names,
                                kind : <#param_tys as ::frei0r_rs::Param>::kind(),
                                explanation : #param_explains,
                            }),*,
                            _ => unreachable!()
                        }
                    }

                    fn param_ref(&self, index : usize) -> ::frei0r_rs::ParamRef<'_> {
                        match index {
                            #(#param_indices =>  <#param_tys as ::frei0r_rs::Param>::as_ref(&self.#param_idents)),*,
                            _ => unreachable!()
                        }
                    }

                    fn param_mut(&mut self, index : usize) -> ::frei0r_rs::ParamMut<'_> {
                        match index {
                            #(#param_indices =>  <#param_tys as ::frei0r_rs::Param>::as_mut(&mut self.#param_idents)),*,
                            _ => unreachable!()
                        }
                    }
                }
            }
        })
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

