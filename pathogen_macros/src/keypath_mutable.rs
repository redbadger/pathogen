use darling::{
    ast::{self, Fields},
    FromAttributes, FromDeriveInput, FromField, FromVariant,
};
use proc_macro2::{Literal, TokenStream};
use proc_macro_error::abort_call_site;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, DeriveInput, Ident};

use crate::{field_name, ContainerSerdeAttrs, ItemSerdeAtrs};

pub(crate) fn keypath_mutable_impl(input: &DeriveInput) -> TokenStream {
    let input = match KeyPathMutableType::from_derive_input(input) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors();
        }
    };

    quote!(#input)
}

#[derive(FromDeriveInput, Debug)]
#[darling(forward_attrs(serde, keypath_mutable))]
struct KeyPathMutableType {
    ident: Ident,
    data: ast::Data<KeyPathMutableEnumVariant, KeyPathMutableStructField>,
    attrs: Vec<syn::Attribute>,
}

#[derive(FromField, Debug)]
#[darling(forward_attrs(serde, keypath_mutable))]
struct KeyPathMutableStructField {
    ident: Option<Ident>,
    attrs: Vec<syn::Attribute>,
}

#[derive(FromAttributes, Debug)]
#[darling(attributes(keypath_mutable))]
struct KeyPathMutableAttrs {
    /// Directs the macro to use "direct dispatch".
    ///
    /// This means the keypath won't include elements for identifying which
    /// variant to target. Instead, we always apply to the current variant.
    direct_dispatch: Option<bool>,

    skip: Option<bool>,
    skip_all: Option<bool>,
}

impl KeyPathMutableAttrs {
    fn should_dispatch_directly(&self) -> bool {
        self.direct_dispatch.unwrap_or(false)
    }

    fn should_skip(&self) -> bool {
        self.skip.unwrap_or(false)
    }

    fn should_skip_all(&self) -> bool {
        self.skip_all.unwrap_or(false)
    }
}

#[derive(FromVariant, Debug)]
#[darling(forward_attrs(serde, keypath_mutable))]
struct KeyPathMutableEnumVariant {
    ident: Ident,
    fields: darling::ast::Fields<KeyPathMutableStructField>,
    attrs: Vec<syn::Attribute>,
}

impl KeyPathMutableEnumVariant {
    fn is_tuple_variant(&self) -> bool {
        self.fields.iter().any(|f| f.ident.is_none())
    }
}

impl ToTokens for KeyPathMutableType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(fields) = self.data.as_ref().take_struct() {
            return Self::derive_struct(tokens, &self.ident, fields, &self.attrs);
        }

        if let Some(variants) = self.data.as_ref().take_enum() {
            return Self::derive_enum(tokens, &self.ident, variants, &self.attrs);
        }

        abort_call_site!("derive(KeyPathMutable) only supports structs");
    }
}

impl KeyPathMutableType {
    fn derive_struct(
        tokens: &mut TokenStream,
        ident: &Ident,
        fields: Fields<&KeyPathMutableStructField>,
        attrs: &[syn::Attribute],
    ) {
        let crate_name = super::crate_name();
        let container_attrs = ContainerSerdeAttrs::from_attributes(attrs);
        let kpm_attrs = KeyPathMutableAttrs::from_attributes(attrs);
        let skip_all = kpm_attrs
            .ok()
            .map(|it| it.should_skip_all())
            .unwrap_or(false);

        let match_arms: Vec<_> = fields
            .into_iter()
            .enumerate()
            .filter_map(|(i, f)| {
                if KeyPathMutableAttrs::from_attributes(&f.attrs)
                    .unwrap()
                    .should_skip()
                {
                    return None;
                };

                Some(if let Some(ident) = f.ident.as_ref() {
                    // Structs
                    let field_attrs = ItemSerdeAtrs::from_attributes(&f.attrs);
                    let ident_name = field_name(ident, &container_attrs, &field_attrs);

                    quote! { #ident_name => self.#ident.patch_keypath(&keys[1..], patch) }
                } else {
                    // Tuple structs
                    let lit = Literal::usize_unsuffixed(i);
                    let lit_name = i.to_string();

                    quote! { #lit_name => self.#lit.patch_keypath(&keys[1..], patch) }
                })
            })
            .collect();

        let fields_match = if skip_all || match_arms.is_empty() {
            quote! {
                Err(#crate_name::KeyPathError::unknown_field::<#ident>(key))
            }
        } else {
            quote! {
                match key {
                    #( #match_arms ),*,
                    _ => Err(#crate_name::KeyPathError::unknown_field::<#ident>(key)),
                }
            }
        };

        tokens.extend(quote! {
            impl #crate_name::KeyPathMutable for #ident {
                fn patch_keypath(&mut self, keys: &[#crate_name::KeyPathElement], patch: #crate_name::Patch) -> Result<(), #crate_name::KeyPathError> {

                    if keys.is_empty() {
                        return if let #crate_name::Patch::Update { value, .. } = patch {
                            *self = serde_json::from_value(value).map_err(#crate_name::KeyPathError::from_deserialization_error::<#ident>)?;
                            Ok(())
                        } else {
                            Err(#crate_name::KeyPathError::cannot_splice_type::<#ident>())
                        };
                    }

                    let #crate_name::KeyPathElement::Field { key } = keys[0] else {
                        return Err(#crate_name::KeyPathError::must_mutate_struct_with_field::<#ident>());
                    };

                    #fields_match
                }
            }
        })
    }

    fn derive_enum(
        tokens: &mut TokenStream,
        ident: &Ident,
        variants: Vec<&KeyPathMutableEnumVariant>,
        attrs: &[syn::Attribute],
    ) {
        let crate_name = super::crate_name();
        let serde_attrs = ContainerSerdeAttrs::from_attributes(attrs);
        let kpm_attrs = KeyPathMutableAttrs::from_attributes(attrs).unwrap();
        let dispatch_directly = kpm_attrs.should_dispatch_directly();

        let dispatch = if dispatch_directly {
            let match_arms = variants.into_iter().map(|variant| {
                let kpm_attrs = KeyPathMutableAttrs::from_attributes(&variant.attrs).unwrap();
                if kpm_attrs.should_skip() || kpm_attrs.should_skip_all() {
                    abort_call_site!("skipping variants is not supported with direct dispatch");
                }

                if variant.is_tuple_variant() {
                    Self::direct_tuple_variant_match_arm(variant)
                } else {
                    abort_call_site!("direct dispatch is only supported on tuple variants");
                }
            });

            quote! {
                match self {
                    #(#match_arms),*
                }
            }
        } else {
            let match_arms = variants.into_iter().filter_map(|variant| {
                let kpm_attrs = KeyPathMutableAttrs::from_attributes(&variant.attrs).unwrap();
                if kpm_attrs.should_skip() || variant.fields.is_empty() {
                    return None;
                }

                let skip_all = kpm_attrs.should_skip_all();

                Some(if variant.is_tuple_variant() {
                    Self::tuple_variant_match_arm(variant, skip_all, &serde_attrs)
                } else {
                    Self::struct_variant_match_arm(variant, skip_all, &serde_attrs)
                })
            });

            let match_statement = if kpm_attrs.should_skip_all() {
                quote! {
                    Err(#crate_name::KeyPathError::unknown_variant_or_field::<#ident>(variant, field_name))
                }
            } else {
                quote! {
                    match self {
                        #(#match_arms),*
                        _ => Err(#crate_name::KeyPathError::unknown_variant_or_field::<#ident>(variant, field_name)),
                    }
                }
            };

            quote! {
                let #crate_name::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                    return Err(#crate_name::KeyPathError::must_mutate_enum_with_variant::<#ident>());
                };

                let #crate_name::KeyPathElement::Field { key: field_name } = keys[1] else {
                    return Err(#crate_name::KeyPathError::must_mutate_enum_variant_with_field::<#ident>(variant));
                };

                #match_statement
            }
        };

        tokens.extend(quote! {
            impl #crate_name::KeyPathMutable for #ident {
                fn patch_keypath(&mut self, keys: &[#crate_name::KeyPathElement], patch: #crate_name::Patch) -> Result<(), #crate_name::KeyPathError> {
                    if keys.is_empty() {
                        return if let #crate_name::Patch::Update { value, .. } = patch {
                            *self = serde_json::from_value(value).map_err(#crate_name::KeyPathError::from_deserialization_error::<#ident>)?;
                            Ok(())
                        } else {
                            Err(#crate_name::KeyPathError::cannot_splice_type::<#ident>())
                        };
                    }

                    #dispatch
                }
            }
        });
    }

    fn tuple_variant_match_arm(
        variant: &KeyPathMutableEnumVariant,
        skip_all: bool,
        serde_attrs: &Result<ContainerSerdeAttrs, darling::Error>,
    ) -> TokenStream {
        let crate_name = super::crate_name();
        let variant_name = &variant.ident;
        let variant_attrs = ItemSerdeAtrs::from_attributes(&variant.attrs);
        let variant_name_str = field_name(variant_name, serde_attrs, &variant_attrs);
        let match_arms: Vec<_> = variant
            .fields
            .iter()
            .enumerate()
            .filter_map(Self::tuple_variant_field_match_arm)
            .collect();
        let element_name_bindings = variant
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| Self::tuple_variant_field_binding(i, f));

        if skip_all || match_arms.is_empty() {
            let element_name_bindings = variant.fields.iter().map(|_| quote! { _ });

            return quote! {
                Self::#variant_name(#(#element_name_bindings),*) if variant == #variant_name_str => {
                    Err(#crate_name::KeyPathError::unknown_variant_or_field::<Self>(#variant_name_str, field_name))
                }
            };
        }

        quote! {
            Self::#variant_name(#(#element_name_bindings),*) if variant == #variant_name_str => match field_name {
                #(#match_arms),*,
                _ => Err(#crate_name::KeyPathError::unknown_variant_or_field::<Self>(#variant_name_str, field_name))
            }
        }
    }

    fn tuple_variant_field_match_arm(
        field: (usize, &KeyPathMutableStructField),
    ) -> Option<TokenStream> {
        let keypathmutable_attrs = KeyPathMutableAttrs::from_attributes(&field.1.attrs);
        if keypathmutable_attrs.is_ok_and(|a| a.should_skip()) {
            return None;
        }

        let value_ident = Ident::new(&format!("value{}", field.0), field.1.ident.span());
        let index_str = field.0.to_string();

        Some(quote! {
            #index_str => #value_ident.patch_keypath(&keys[2..], patch)
        })
    }

    fn tuple_variant_field_binding(index: usize, field: &KeyPathMutableStructField) -> Ident {
        let keypathmutable_attrs = KeyPathMutableAttrs::from_attributes(&field.attrs);
        if keypathmutable_attrs.is_ok_and(|a| a.should_skip()) {
            Ident::new(&format!("_value{}", index), field.ident.span())
        } else {
            Ident::new(&format!("value{}", index), field.ident.span())
        }
    }

    fn struct_variant_match_arm(
        variant: &KeyPathMutableEnumVariant,
        skip_all: bool,
        serde_attrs: &Result<ContainerSerdeAttrs, darling::Error>,
    ) -> TokenStream {
        let crate_name = super::crate_name();
        let variant_name = &variant.ident;
        let variant_attrs = ItemSerdeAtrs::from_attributes(&variant.attrs);
        let variant_container_attrs = ContainerSerdeAttrs::from_attributes(&variant.attrs);
        let variant_name_str = field_name(variant_name, serde_attrs, &variant_attrs);
        let match_arms: Vec<_> = variant
            .fields
            .iter()
            .filter_map(|f| Self::struct_variant_field_match_arm(f, &variant_container_attrs))
            .collect();

        let field_name_bindings = variant
            .fields
            .iter()
            .map(Self::struct_variant_field_binding);

        if skip_all || match_arms.is_empty() {
            return quote! {
                Self::#variant_name { .. } if variant == #variant_name_str => {
                    Err(#crate_name::KeyPathError::unknown_variant_or_field::<Self>(#variant_name_str, field_name))
                }
            };
        }

        quote! {
            Self::#variant_name { #(#field_name_bindings),* } if variant == #variant_name_str => match field_name {
                #(#match_arms),*,
                _ => Err(#crate_name::KeyPathError::unknown_variant_or_field::<Self>(#variant_name_str, field_name))
            }
        }
    }

    fn struct_variant_field_match_arm(
        field: &KeyPathMutableStructField,
        serde_attrs: &Result<ContainerSerdeAttrs, darling::Error>,
    ) -> Option<TokenStream> {
        let keypathmutable_attrs = KeyPathMutableAttrs::from_attributes(&field.attrs);
        if keypathmutable_attrs.is_ok_and(|a| a.should_skip()) {
            return None;
        }

        let ident = field
            .ident
            .as_ref()
            .expect("no ident for struct variant field");

        let field_attrs = ItemSerdeAtrs::from_attributes(&field.attrs);
        let field_name_str = field_name(ident, serde_attrs, &field_attrs);

        Some(quote! {
            #field_name_str => #ident.patch_keypath(&keys[2..], patch)
        })
    }

    fn struct_variant_field_binding(field: &KeyPathMutableStructField) -> TokenStream {
        let keypathmutable_attrs = KeyPathMutableAttrs::from_attributes(&field.attrs);
        if keypathmutable_attrs.is_ok_and(|a| a.should_skip()) {
            field
                .ident
                .as_ref()
                .map(|oid| {
                    quote! {
                        #oid: _
                    }
                })
                .expect("Field should have an identifier")
        } else {
            let oid = &field.ident;
            quote! { #oid }
        }
    }

    fn direct_tuple_variant_match_arm(variant: &KeyPathMutableEnumVariant) -> TokenStream {
        let variant_name = &variant.ident;
        if variant.fields.len() != 1 {
            abort_call_site!(
                "tuple variants must have exactly one element to support direct dispatch"
            );
        }

        quote! {
            Self::#variant_name(value) => value.patch_keypath(keys, patch)
        }
    }
}

#[cfg(test)]
#[path = "keypath_mutable.test.rs"]
mod tests;
