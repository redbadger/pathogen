use darling::FromDeriveInput;
use quote::quote;
use syn::parse_str;

use super::KeyPathMutableType;

fn pretty_print(ts: &proc_macro2::TokenStream) -> String {
    if let Ok(file) = syn::parse_file(&ts.to_string()) {
        prettyplease::unparse(&file)
    } else {
        panic!("Invalid output to pretty_print: {:?}", ts.to_string())
    }
}

#[test]
fn struct_with_one_field() {
    let input = r#"
            #[derive(Navigable)]
            struct MyStruct {
                a: usize,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyStruct {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyStruct>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyStruct>())
                };
            }
            let key_path::KeyPathElement::Field { key } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_struct_with_field::<MyStruct>(),
                );
            };
            match key {
                "a" => self.a.patch_keypath(&keys[1..], patch),
                _ => Err(key_path::KeyPathError::unknown_field::<MyStruct>(key)),
            }
        }
    }
    "###);
}

#[test]
fn struct_with_one_skipped_field() {
    let input = r#"
            #[derive(Navigable)]
            struct MyStruct {
                #[keypath_mutable(skip)]
                a: usize,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyStruct {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyStruct>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyStruct>())
                };
            }
            let key_path::KeyPathElement::Field { key } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_struct_with_field::<MyStruct>(),
                );
            };
            Err(key_path::KeyPathError::unknown_field::<MyStruct>(key))
        }
    }
    "###);
}

#[test]
fn struct_with_skip_all() {
    let input = r#"
            #[derive(Navigable)]
            #[keypath_mutable(skip_all)]
            struct MyStruct {
                a: usize,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyStruct {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyStruct>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyStruct>())
                };
            }
            let key_path::KeyPathElement::Field { key } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_struct_with_field::<MyStruct>(),
                );
            };
            Err(key_path::KeyPathError::unknown_field::<MyStruct>(key))
        }
    }
    "###);
}

#[test]
fn newtype_struct() {
    let input = r#"
            struct MyNumber(usize);
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyNumber {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyNumber>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyNumber>())
                };
            }
            let key_path::KeyPathElement::Field { key } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_struct_with_field::<MyNumber>(),
                );
            };
            match key {
                "0" => self.0.patch_keypath(&keys[1..], patch),
                _ => Err(key_path::KeyPathError::unknown_field::<MyNumber>(key)),
            }
        }
    }
    "###);
}

#[test]
fn struct_with_multiple_fields() {
    let input = r#"
            #[derive(Navigable)]
            struct MyStruct {
                a: usize,
                b: String,
                c: f64,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyStruct {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyStruct>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyStruct>())
                };
            }
            let key_path::KeyPathElement::Field { key } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_struct_with_field::<MyStruct>(),
                );
            };
            match key {
                "a" => self.a.patch_keypath(&keys[1..], patch),
                "b" => self.b.patch_keypath(&keys[1..], patch),
                "c" => self.c.patch_keypath(&keys[1..], patch),
                _ => Err(key_path::KeyPathError::unknown_field::<MyStruct>(key)),
            }
        }
    }
    "###);
}

#[test]
fn struct_with_multiple_fields_and_rename() {
    let input = r#"
            #[derive(Navigable)]
            #[serde(rename_all = "camelCase")]
            struct MyStruct {
                long_field: usize,
                even_longer_field: String,
                and_one_more: f64,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyStruct {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyStruct>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyStruct>())
                };
            }
            let key_path::KeyPathElement::Field { key } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_struct_with_field::<MyStruct>(),
                );
            };
            match key {
                "longField" => self.long_field.patch_keypath(&keys[1..], patch),
                "evenLongerField" => self.even_longer_field.patch_keypath(&keys[1..], patch),
                "andOneMore" => self.and_one_more.patch_keypath(&keys[1..], patch),
                _ => Err(key_path::KeyPathError::unknown_field::<MyStruct>(key)),
            }
        }
    }
    "###);
}

#[test]
fn enum_with_no_data() {
    let input = r#"
            enum BasicEnum {
                First,
                Second,
                Third,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for BasicEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<BasicEnum>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<BasicEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<BasicEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        BasicEnum,
                    >(variant),
                );
            };
            match self {
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            BasicEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}

#[test]
fn enum_with_all_trimmings() {
    let input = r#"
            enum ExhaustingEnum {
                First(usize),
                Second { field: String },
                Third(usize, String),
                Fourth { field1: usize, field2: String },
                Fifth(SimpleStruct),
                Sixth { field: SimpleStruct },
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for ExhaustingEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<
                                ExhaustingEnum,
                            >,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<ExhaustingEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<ExhaustingEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        ExhaustingEnum,
                    >(variant),
                );
            };
            match self {
                Self::First(value0) if variant == "First" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("First", field_name),
                            )
                        }
                    }
                }
                Self::Second { field } if variant == "Second" => {
                    match field_name {
                        "field" => field.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Second", field_name),
                            )
                        }
                    }
                }
                Self::Third(value0, value1) if variant == "Third" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        "1" => value1.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Third", field_name),
                            )
                        }
                    }
                }
                Self::Fourth { field1, field2 } if variant == "Fourth" => {
                    match field_name {
                        "field1" => field1.patch_keypath(&keys[2..], patch),
                        "field2" => field2.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Fourth", field_name),
                            )
                        }
                    }
                }
                Self::Fifth(value0) if variant == "Fifth" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Fifth", field_name),
                            )
                        }
                    }
                }
                Self::Sixth { field } if variant == "Sixth" => {
                    match field_name {
                        "field" => field.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Sixth", field_name),
                            )
                        }
                    }
                }
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            ExhaustingEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}

#[test]
fn enum_with_all_trimmings_and_serde() {
    let input = r#"
            #[serde(rename_all = "camelCase")]
            enum ExhaustingEnum {
                FirstThing(usize),
                #[serde(rename_all = "camelCase")]
                SecondThing { long_field: String },
                ThirdOption(usize, String),
                #[serde(rename_all = "camelCase")]
                FourthKind { long_field: usize, #[serde(rename = "longer_field")] even_longer_field: String },
                FifthCleverThing(SimpleStruct),
                Sixth { field: SimpleStruct },
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for ExhaustingEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<
                                ExhaustingEnum,
                            >,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<ExhaustingEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<ExhaustingEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        ExhaustingEnum,
                    >(variant),
                );
            };
            match self {
                Self::FirstThing(value0) if variant == "firstThing" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("firstThing", field_name),
                            )
                        }
                    }
                }
                Self::SecondThing { long_field } if variant == "secondThing" => {
                    match field_name {
                        "longField" => long_field.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("secondThing", field_name),
                            )
                        }
                    }
                }
                Self::ThirdOption(value0, value1) if variant == "thirdOption" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        "1" => value1.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("thirdOption", field_name),
                            )
                        }
                    }
                }
                Self::FourthKind {
                    long_field,
                    even_longer_field,
                } if variant == "fourthKind" => {
                    match field_name {
                        "longField" => long_field.patch_keypath(&keys[2..], patch),
                        "longer_field" => even_longer_field.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("fourthKind", field_name),
                            )
                        }
                    }
                }
                Self::FifthCleverThing(value0) if variant == "fifthCleverThing" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("fifthCleverThing", field_name),
                            )
                        }
                    }
                }
                Self::Sixth { field } if variant == "sixth" => {
                    match field_name {
                        "field" => field.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("sixth", field_name),
                            )
                        }
                    }
                }
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            ExhaustingEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}

#[test]
fn struct_with_a_skip() {
    let input = r#"
            struct MyStruct {
                long_field: usize,
                #[keypath_mutable(skip)]
                even_longer_field: String,
                and_one_more: f64,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyStruct {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyStruct>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyStruct>())
                };
            }
            let key_path::KeyPathElement::Field { key } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_struct_with_field::<MyStruct>(),
                );
            };
            match key {
                "long_field" => self.long_field.patch_keypath(&keys[1..], patch),
                "and_one_more" => self.and_one_more.patch_keypath(&keys[1..], patch),
                _ => Err(key_path::KeyPathError::unknown_field::<MyStruct>(key)),
            }
        }
    }
    "###);
}

#[test]
fn enum_with_a_skip() {
    let input = r#"
            enum MyEnum {
                First,
                #[keypath_mutable(skip)]
                Second(usize),
                Third(isize),
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyEnum>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<MyEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        MyEnum,
                    >(variant),
                );
            };
            match self {
                Self::Third(value0) if variant == "Third" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Third", field_name),
                            )
                        }
                    }
                }
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            MyEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}

#[test]
fn enum_with_skip_all() {
    let input = r#"
            enum MyEnum {
                First,
                #[keypath_mutable(skip_all)]
                Second(usize),
                Third(isize),
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyEnum>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<MyEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        MyEnum,
                    >(variant),
                );
            };
            match self {
                Self::Second(_) if variant == "Second" => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            Self,
                        >("Second", field_name),
                    )
                }
                Self::Third(value0) if variant == "Third" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Third", field_name),
                            )
                        }
                    }
                }
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            MyEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}

#[test]
fn enum_with_a_top_level_skip_all() {
    let input = r#"
            #[keypath_mutable(skip_all)]
            enum MyEnum {
                First,
                Second(usize),
                Third(isize),
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyEnum>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<MyEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        MyEnum,
                    >(variant),
                );
            };
            Err(
                key_path::KeyPathError::unknown_variant_or_field::<
                    MyEnum,
                >(variant, field_name),
            )
        }
    }
    "###);
}

#[test]
fn enum_with_a_skip_in_a_struct_variant() {
    let input = r#"
            enum MyEnum {
                First,
                Second { a: usize, #[keypath_mutable(skip)] b: String },
                Third(isize),
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyEnum>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<MyEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        MyEnum,
                    >(variant),
                );
            };
            match self {
                Self::Second { a, b: _ } if variant == "Second" => {
                    match field_name {
                        "a" => a.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Second", field_name),
                            )
                        }
                    }
                }
                Self::Third(value0) if variant == "Third" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Third", field_name),
                            )
                        }
                    }
                }
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            MyEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}

#[test]
fn enum_with_a_skip_in_a_tuple_variant() {
    let input = r#"
            enum MyEnum {
                First,
                Second(usize, #[keypath_mutable(skip)] String),
                Third(isize),
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyEnum>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<MyEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        MyEnum,
                    >(variant),
                );
            };
            match self {
                Self::Second(value0, _value1) if variant == "Second" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Second", field_name),
                            )
                        }
                    }
                }
                Self::Third(value0) if variant == "Third" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Third", field_name),
                            )
                        }
                    }
                }
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            MyEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}

#[test]
fn enum_with_all_items_skipped_in_a_tuple_variant() {
    let input = r#"
            enum MyEnum {
                First,
                Second(#[keypath_mutable(skip)] usize, #[keypath_mutable(skip)] String),
                Third(isize),
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyEnum>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<MyEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        MyEnum,
                    >(variant),
                );
            };
            match self {
                Self::Second(_, _) if variant == "Second" => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            Self,
                        >("Second", field_name),
                    )
                }
                Self::Third(value0) if variant == "Third" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Third", field_name),
                            )
                        }
                    }
                }
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            MyEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}

#[test]
fn enum_with_all_fields_skipped_in_a_struct_variant() {
    let input = r#"
            enum MyEnum {
                First,
                Second { #[keypath_mutable(skip)] a: usize, #[keypath_mutable(skip)] b: String },
                Third(isize),
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyEnum>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<MyEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        MyEnum,
                    >(variant),
                );
            };
            match self {
                Self::Second { .. } if variant == "Second" => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            Self,
                        >("Second", field_name),
                    )
                }
                Self::Third(value0) if variant == "Third" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Third", field_name),
                            )
                        }
                    }
                }
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            MyEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}

#[test]
fn enum_with_skipped_all_on_a_struct_variant() {
    let input = r#"
            enum MyEnum {
                First,
                #[keypath_mutable(skip_all)]
                Second { a: usize, b: String },
                Third(isize),
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = KeyPathMutableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::KeyPathMutable for MyEnum {
        fn patch_keypath(
            &mut self,
            keys: &[key_path::KeyPathElement],
            patch: key_path::Patch,
        ) -> Result<(), key_path::KeyPathError> {
            if keys.is_empty() {
                return if let key_path::Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(
                            key_path::KeyPathError::from_deserialization_error::<MyEnum>,
                        )?;
                    Ok(())
                } else {
                    Err(key_path::KeyPathError::cannot_splice_type::<MyEnum>())
                };
            }
            let key_path::KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_with_variant::<MyEnum>(),
                );
            };
            let key_path::KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(
                    key_path::KeyPathError::must_mutate_enum_variant_with_field::<
                        MyEnum,
                    >(variant),
                );
            };
            match self {
                Self::Second { .. } if variant == "Second" => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            Self,
                        >("Second", field_name),
                    )
                }
                Self::Third(value0) if variant == "Third" => {
                    match field_name {
                        "0" => value0.patch_keypath(&keys[2..], patch),
                        _ => {
                            Err(
                                key_path::KeyPathError::unknown_variant_or_field::<
                                    Self,
                                >("Third", field_name),
                            )
                        }
                    }
                }
                _ => {
                    Err(
                        key_path::KeyPathError::unknown_variant_or_field::<
                            MyEnum,
                        >(variant, field_name),
                    )
                }
            }
        }
    }
    "###);
}
