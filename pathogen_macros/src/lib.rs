mod keypath_mutable;
mod navigable;

use std::env;

use darling::FromAttributes;
use proc_macro::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, Ident};

use keypath_mutable::keypath_mutable_impl;
use navigable::navigable_impl;

#[proc_macro_derive(Navigable)]
#[proc_macro_error]
pub fn navigable(input: TokenStream) -> TokenStream {
    navigable_impl(&parse_macro_input!(input)).into()
}

#[proc_macro_derive(KeyPathMutable, attributes(keypath_mutable))]
#[proc_macro_error]
pub fn keypath_mutable(input: TokenStream) -> TokenStream {
    keypath_mutable_impl(&parse_macro_input!(input)).into()
}

fn crate_name() -> proc_macro2::TokenStream {
    let in_self = env::var("CARGO_PKG_NAME").unwrap() == "pathogen";
    if in_self {
        quote! { crate }
    } else {
        quote! { key_path }
    }
}

/// Used for attributes on structs or enums
#[derive(FromAttributes, Debug)]
#[darling(attributes(serde), allow_unknown_fields)]
struct ContainerSerdeAttrs {
    rename_all: Option<String>,
    tag: Option<String>,
    content: Option<String>,
    untagged: Option<bool>,
}

/// Used for attributes on fields or variants
#[derive(FromAttributes, Debug)]
#[darling(attributes(serde), allow_unknown_fields)]
struct ItemSerdeAtrs {
    rename: Option<String>,
}

enum VariantTagType {
    External,
    Internal,
    Adjacent,
    Untagged,
}

fn field_name(
    ident: &Ident,
    container_serde_attrs: &Result<ContainerSerdeAttrs, darling::Error>,
    item_serde_attrs: &Result<ItemSerdeAtrs, darling::Error>,
) -> String {
    if let Ok(item_attrs) = item_serde_attrs {
        if let Some(name) = &item_attrs.rename {
            return name.to_string();
        }
    }

    let Ok(conatiner_attrs) = container_serde_attrs else {
        return ident.to_string();
    };

    let ident_str = ident.to_string();

    match conatiner_attrs.rename_all.as_deref() {
        None => ident_str,
        Some("camelCase") => {
            let mut upcase = false;
            let mut renamed = ident_str[0..1].to_lowercase();

            for chr in ident_str[1..].chars() {
                if chr == '_' {
                    upcase = true;
                    continue;
                }

                if upcase {
                    renamed.push_str(&chr.to_uppercase().to_string());
                    upcase = false;
                } else {
                    renamed.push(chr);
                }
            }

            renamed
        }
        Some(other) => {
            abort_call_site!("Unsupported rename_all value: {}", other);
        }
    }
}

fn tag_type_from_serde_attrs(
    attrs: &Result<ContainerSerdeAttrs, darling::Error>,
) -> VariantTagType {
    let Ok(attrs) = attrs else {
        return VariantTagType::External;
    };
    if attrs.content.is_some() {
        VariantTagType::Adjacent
    } else if attrs.tag.is_some() {
        VariantTagType::Internal
    } else if attrs.untagged.unwrap_or(false) {
        VariantTagType::Untagged
    } else {
        VariantTagType::External
    }
}
