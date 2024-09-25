use darling::{
    ast::{self, Fields},
    FromAttributes, FromDeriveInput, FromField, FromVariant,
};
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{quote, ToTokens};
use syn::{DeriveInput, Ident};

use crate::{
    field_name, tag_type_from_serde_attrs, ContainerSerdeAttrs, ItemSerdeAtrs, VariantTagType,
};

pub(crate) fn navigable_impl(input: &DeriveInput) -> TokenStream {
    let input = match NavigableType::from_derive_input(input) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors();
        }
    };

    quote!(#input)
}

#[derive(FromDeriveInput, Debug)]
#[darling(forward_attrs(serde))]
struct NavigableType {
    ident: Ident,
    data: ast::Data<NavigableEnumVariant, NavigableStructField>,
    attrs: Vec<syn::Attribute>,
}

#[derive(FromField, Debug)]
#[darling(forward_attrs(serde))]
struct NavigableStructField {
    ident: Option<Ident>,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

#[derive(FromVariant, Debug)]
#[darling(forward_attrs(serde))]
struct NavigableEnumVariant {
    ident: Ident,
    fields: darling::ast::Fields<NavigableStructField>,
    attrs: Vec<syn::Attribute>,
}

impl NavigableEnumVariant {
    fn is_tuple_variant(&self) -> bool {
        self.fields.iter().any(|f| f.ident.is_none())
    }
}

impl ToTokens for NavigableType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(fields) = self.data.as_ref().take_struct() {
            return Self::derive_struct(tokens, &self.ident, fields, &self.attrs);
        }

        if let Some(variants) = self.data.as_ref().take_enum() {
            return Self::derive_enum(tokens, &self.ident, variants, &self.attrs);
        }

        abort_call_site!("derive(Navigable) only supports structs and enums with struct variants.");
    }
}

impl NavigableType {
    fn derive_struct(
        tokens: &mut TokenStream,
        path_source: &Ident,
        fields: Fields<&NavigableStructField>,
        attrs: &[syn::Attribute],
    ) {
        let names_and_types: Vec<_> = fields
            .into_iter()
            .map(|f| {
                let ident = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                let attrs = f.attrs.as_slice();

                (ident, ty, attrs)
            })
            .collect();
        let serde_attrs = ContainerSerdeAttrs::from_attributes(attrs);

        let reflection_type_name = Self::reflection_type_name(path_source);
        let (field_declarations, field_values) =
            Self::reflection_type_fields(&names_and_types, &serde_attrs);

        let crate_name = super::crate_name();

        tokens.extend(quote! {
            impl #crate_name::Navigable for #path_source {
                type Reflection<Root> = #reflection_type_name<Root>;

                fn append_to_keypath<Root>(path: &#crate_name::KeyPath<Root, Self>) -> Self::Reflection<Root>
                where
                    Root: Sized,
                {
                    #reflection_type_name {
                        #( #field_values ),*
                    }
                }
            }
        });

        tokens.extend(quote! {
            pub struct #reflection_type_name<Root> {
                #(#field_declarations),*
            }
        });
    }

    fn derive_enum(
        tokens: &mut TokenStream,
        path_source: &Ident,
        variants: Vec<&NavigableEnumVariant>,
        attrs: &[syn::Attribute],
    ) {
        let serde_attrs = ContainerSerdeAttrs::from_attributes(attrs);

        let reflection_type_name = Self::reflection_type_name(path_source);
        let (field_declarations, field_values): (Vec<_>, Vec<_>) = variants
            .iter()
            .map(|v| {
                (
                    Self::derive_enum_variant_field_declaration(tokens, path_source, v),
                    Self::derive_enum_variant_field_value(v, &serde_attrs),
                )
            })
            .unzip();

        let crate_name = super::crate_name();

        tokens.extend(quote! {
            #[allow(non_snake_case)]
            pub struct #reflection_type_name<Root> {
                #(#field_declarations),*
            }
        });

        tokens.extend(quote! {
            impl #crate_name::Navigable for #path_source {
                type Reflection<Root> = #reflection_type_name<Root>;

                fn append_to_keypath<Root>(path: &#crate_name::KeyPath<Root, Self>) -> Self::Reflection<Root>
                where
                    Root: Sized,
                {
                    #reflection_type_name {
                        #( #field_values ),*
                    }
                }
            }
        });
    }

    fn derive_enum_variant_field_declaration(
        tokens: &mut TokenStream,
        type_name: &Ident,
        variant: &NavigableEnumVariant,
    ) -> TokenStream {
        if variant.is_tuple_variant() {
            Self::derive_enum_tuple_variant(variant)
        } else {
            Self::derive_enum_struct_variant(tokens, type_name, variant)
        }
    }

    /// Derive and output the definition of the reflection type for a tuple variant of an enum.
    /// Return the derived type name as a TokenStream (to be compatible with 'derive_enum_struct_variant)
    ///
    /// fields are the unnamed fields of the tuple variant (e.g. `.0` and `.1` in `VariantOne(usize, String)`
    fn derive_enum_tuple_variant(variant: &NavigableEnumVariant) -> TokenStream {
        let crate_name = super::crate_name();
        let variant_name = &variant.ident;

        let tuple_items = variant.fields.iter().map(|f| {
            let ty = &f.ty;
            quote! {
                #crate_name::KeyPath<Root, #ty>
            }
        });

        quote! {
            pub #variant_name: (#(#tuple_items,)*)
        }
    }

    /// Derive and output the definition of the reflection type for a struct variant of an enum.
    /// Return the derived type name as a TokenStream (to be compatible with 'derive_enum_tuple_variant)
    ///
    /// fields are the named fields of the struct variant (the "anonymous struct")
    fn derive_enum_struct_variant(
        tokens: &mut TokenStream,
        type_name: &Ident,
        variant: &NavigableEnumVariant,
    ) -> TokenStream {
        let fields: Vec<_> = variant
            .fields
            .iter()
            .map(|f| {
                let ident = f.ident.as_ref().unwrap();
                let ty = &f.ty;
                let attrs = f.attrs.as_slice();

                (ident, ty, attrs)
            })
            .collect();

        let reflection_type_name = Ident::new(
            &format!("{}KeyPathReflectionVariant{}", type_name, variant.ident),
            variant.ident.span(),
        );
        let serde_attrs = ContainerSerdeAttrs::from_attributes(&variant.attrs);

        let (field_declarations, field_values) =
            Self::reflection_type_fields(&fields, &serde_attrs);
        let crate_name = super::crate_name();

        tokens.extend(quote! {
            pub struct #reflection_type_name<Root> {
                #(#field_declarations),*
            }
        });

        tokens.extend(quote! {
            impl<T> #crate_name::Navigable for #reflection_type_name<T> {
                type Reflection<Root> = #reflection_type_name<Root>;

                fn append_to_keypath<Root>(path: &#crate_name::KeyPath<Root, Self>) -> Self::Reflection<Root>
                where
                    Root: Sized,
                {
                    #reflection_type_name {
                        #( #field_values ),*
                    }
                }
            }
        });

        let variant_name = &variant.ident;
        quote! {
            pub #variant_name: #crate_name::KeyPath<Root, #reflection_type_name<Root>>
        }
    }

    fn derive_enum_variant_field_value(
        variant: &NavigableEnumVariant,
        serde_attrs: &Result<ContainerSerdeAttrs, darling::Error>,
    ) -> TokenStream {
        let variant_name = &variant.ident;
        let variant_attrs = ItemSerdeAtrs::from_attributes(&variant.attrs);
        let variant_str = field_name(variant_name, serde_attrs, &variant_attrs);

        let crate_name = super::crate_name();
        let tag_type = match tag_type_from_serde_attrs(serde_attrs) {
            VariantTagType::External => quote!(#crate_name::VariantTagType::External),
            VariantTagType::Internal => quote!(#crate_name::VariantTagType::Internal),
            VariantTagType::Adjacent => quote!(#crate_name::VariantTagType::Adjacent),
            VariantTagType::Untagged => quote!(#crate_name::VariantTagType::Untagged),
        };

        if variant.is_tuple_variant() {
            let variant_paths = variant.fields.iter().enumerate().map(|(field_index, _)| {
                let field_index = field_index.to_string();
                quote! {
                    path.appending(&#crate_name::KeyPath::tuple_variant(
                        #variant_str,
                        #field_index,
                        #tag_type,
                    ))
                }
            });

            quote! {
                #variant_name: ( #( #variant_paths ,)* )
            }
        } else {
            quote! {
                #variant_name: path.appending(
                    &#crate_name::KeyPath::variant(
                        #variant_str,
                        #tag_type,
                    )
                )
            }
        }
    }

    fn reflection_type_fields(
        fields: &[(&Ident, &syn::Type, &[syn::Attribute])],
        serde_attrs: &Result<ContainerSerdeAttrs, darling::Error>,
    ) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let crate_name = super::crate_name();
        let declarations = fields
            .iter()
            .map(|(ident, ty, _)| {
                quote! {
                    pub #ident: #crate_name::KeyPath<Root, #ty>
                }
            })
            .collect();

        let values: Vec<_> = fields
            .iter()
            .map(|(ident, _, attrs)| {
                let field_attrs = ItemSerdeAtrs::from_attributes(attrs);
                let field_str = field_name(ident, serde_attrs, &field_attrs);
                quote! {
                    #ident: path.appending(&#crate_name::KeyPath::field(#field_str))
                }
            })
            .collect();

        (declarations, values)
    }

    fn reflection_type_name(path_source: &Ident) -> Ident {
        Ident::new(
            &format!("{}KeyPathReflection", path_source),
            path_source.span(),
        )
    }
}

#[cfg(test)]
#[path = "navigable.test.rs"]
mod tests;
