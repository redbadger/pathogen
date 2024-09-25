use std::collections::BTreeMap;
use std::{any::type_name, str::FromStr};

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use thiserror::Error;
use uuid::Uuid;

use super::Patch;
use crate::KeyPathElement;

use super::{AsPatch, ChangeOf};

#[derive(Debug, Error)]
pub enum KeyPathError {
    #[error("attempt to mutate inside a `None`")]
    CannotMutateNone,
    #[error("attempt to mutate inside primitive type {type_name}")]
    CannotMutatePrimitiveChildren { type_name: &'static str },
    #[error("attempt to splice type {type_name}")]
    CannotSpliceType { type_name: &'static str },
    #[error("error deserializing type {type_name}: {error}")]
    DeserializationError {
        type_name: &'static str,
        error: serde_json::Error,
    },
    #[error("attempt to mutate enum variant {type_name}::{variant}, but the KeyPathElement was not a field")]
    MustMutateEnumVariantWithField {
        type_name: &'static str,
        variant: &'static str,
    },
    #[error("attempt to mutate type {type_name}, but the KeyPathElement was not a variant")]
    MustMutateEnumWithVariant { type_name: &'static str },
    #[error("attempt to mutate type {type_name}, but the KeyPathElement was not a field")]
    MustMutateStructWithField { type_name: &'static str },
    #[error("attempt to mutate a vector, but the KeyPathElement was not an index")]
    MustMutateVectorWithIndex,
    #[error("attempt to mutate a map, but the KeyPathElement was not a string key")]
    MustMutateMapWithStringKey,
    #[error("attempt to mutate type {type_name} with unknown field: {field}")]
    UnknownField {
        type_name: &'static str,
        field: &'static str,
    },
    #[error("attempt to mutate non-existing key {key}")]
    UnknownStringKey { key: String },
    #[error("attempt to mutate enum {type_name} with unknown variant or field: {variant}.{field}")]
    UnknownVariantOrField {
        type_name: &'static str,
        variant: &'static str,
        field: &'static str,
    },
}

impl KeyPathError {
    pub fn cannot_splice_type<T>() -> Self {
        KeyPathError::CannotSpliceType {
            type_name: type_name::<T>(),
        }
    }

    pub fn from_deserialization_error<T>(error: serde_json::Error) -> Self {
        KeyPathError::DeserializationError {
            type_name: type_name::<T>(),
            error,
        }
    }

    pub fn must_mutate_enum_with_variant<T>() -> Self {
        KeyPathError::MustMutateEnumWithVariant {
            type_name: type_name::<T>(),
        }
    }

    pub fn must_mutate_enum_variant_with_field<T>(variant: &'static str) -> Self {
        KeyPathError::MustMutateEnumVariantWithField {
            type_name: type_name::<T>(),
            variant,
        }
    }

    pub fn must_mutate_struct_with_field<T>() -> Self {
        KeyPathError::MustMutateStructWithField {
            type_name: type_name::<T>(),
        }
    }

    pub fn unknown_field<T>(field: &'static str) -> Self {
        KeyPathError::UnknownField {
            type_name: type_name::<T>(),
            field,
        }
    }

    pub fn unknown_variant_or_field<T>(variant: &'static str, field: &'static str) -> Self {
        KeyPathError::UnknownVariantOrField {
            type_name: type_name::<T>(),
            variant,
            field,
        }
    }
}

// TODO: consider making this part of Navigable when finished
pub trait KeyPathMutable
where
    Self: Sized + 'static,
{
    /// Mutate by a keypath (as a slice of elements) in a member that is a struct or enum
    //
    // Implementation notes:
    //
    // This can't do a very thorough type checking, because the paths are type erased, but constructing
    // an invalid path should be impossible or at least _very_ difficult
    //
    // If the keypath has multiple keys
    // 1. Take the first key and verify it is the right kind
    // 2. Match on key for all known keys - for enums, verify self is the right variant and if there are no more keys, apply patch to self
    // 3. Call patch_keypath on the matching struct field / variant field with the rest of the keypath
    //
    // If the keypath has a single key
    // 1. Verify key is the right kind - Field for struct, Variant for enum (both macro derived), Index for vector (implemented by hand)
    // 2. Match on key for all known keys - fields or variants
    // 3. Match on Patch type and update self.[key] to deserialised value (type is now known based on Self)
    fn patch_keypath(&mut self, keys: &[KeyPathElement], patch: Patch) -> Result<(), KeyPathError>;

    /// Apply a `ChangeOf<Self>` to self, which will mutate a deeply nested value based on the keypath
    fn apply_change(&mut self, change: &ChangeOf<Self>) {
        self.patch_keypath(&change.key_path().path, change.as_patch())
            .expect("patch failure");
    }
}

impl<T: KeyPathMutable + DeserializeOwned> KeyPathMutable for Vec<T> {
    fn patch_keypath(&mut self, keys: &[KeyPathElement], patch: Patch) -> Result<(), KeyPathError> {
        if keys.is_empty() {
            match patch {
                Patch::Splice {
                    value,
                    start,
                    replace,
                    ..
                } => {
                    let replacements = value
                        .into_iter()
                        .map(|v| serde_json::from_value(v))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(KeyPathError::from_deserialization_error::<T>)?;

                    self.splice(start..(start + replace), replacements);
                }
                Patch::Update { value, .. } => {
                    let replacement: Vec<T> = serde_json::from_value(value)
                        .map_err(KeyPathError::from_deserialization_error::<T>)?;

                    self.splice(.., replacement);
                }
            };
            return Ok(());
        }

        let KeyPathElement::Index { key } = keys[0] else {
            return Err(KeyPathError::MustMutateVectorWithIndex);
        };

        let value = &mut self[key];

        // If there are more keys, recurse
        value.patch_keypath(&keys[1..], patch)
    }
}

impl<K, V> KeyPathMutable for BTreeMap<K, V>
where
    K: DeserializeOwned + FromStr + Ord + ToString + 'static,
    V: KeyPathMutable + DeserializeOwned,
{
    fn patch_keypath(&mut self, keys: &[KeyPathElement], patch: Patch) -> Result<(), KeyPathError> {
        if keys.is_empty() {
            return match patch {
                Patch::Update { value, .. } => {
                    *self = serde_json::from_value(value)
                        .map_err(KeyPathError::from_deserialization_error::<Self>)?;
                    Ok(())
                }
                Patch::Splice { .. } => Err(KeyPathError::CannotSpliceType {
                    type_name: "BTreeMap",
                }),
            };
        }

        let KeyPathElement::StringKey { key } = &keys[0] else {
            return Err(KeyPathError::MustMutateMapWithStringKey);
        };

        let Ok(key) = K::from_str(key) else {
            return Err(KeyPathError::UnknownStringKey { key: key.clone() });
        };

        if keys.len() == 1 {
            if let Patch::Update { value, .. } = patch {
                let value = serde_json::from_value(value)
                    .map_err(KeyPathError::from_deserialization_error::<V>)?;
                self.insert(key, value);
                return Ok(());
            }
        }

        if let Some(value) = self.get_mut(&key) {
            value.patch_keypath(&keys[1..], patch)
        } else {
            Err(KeyPathError::UnknownStringKey {
                key: key.to_string(),
            })
        }
    }
}

impl<T> KeyPathMutable for Option<T>
where
    T: DeserializeOwned + KeyPathMutable + 'static,
{
    fn patch_keypath(&mut self, keys: &[KeyPathElement], patch: Patch) -> Result<(), KeyPathError> {
        if !keys.is_empty() {
            if let Some(inner) = self.as_mut() {
                return inner.patch_keypath(keys, patch);
            }

            return Err(KeyPathError::CannotMutateNone);
        }

        let Patch::Update { value, .. } = patch else {
            return Err(KeyPathError::cannot_splice_type::<Option<T>>());
        };

        let value: Option<T> = serde_json::from_value(value)
            .map_err(KeyPathError::from_deserialization_error::<Option<T>>)?;

        *self = value;
        Ok(())
    }
}

macro_rules! keypath_mutable_impl {
    ($($t:ty)*) => ($(
        impl KeyPathMutable for $t {
            fn patch_keypath(&mut self, keys: &[KeyPathElement], patch: Patch) -> Result<(), KeyPathError> {

                if !keys.is_empty() {
                    return Err(KeyPathError::CannotMutatePrimitiveChildren { type_name: type_name::<$t>() });
                }

                let Patch::Update { value, .. } = patch else {
                    return Err(KeyPathError::cannot_splice_type::<$t>());
                };

                let value: $t = serde_json::from_value(value)
                    .map_err(KeyPathError::from_deserialization_error::<$t>)?;

                *self = value;
                Ok(())
            }
        }
    )*);
}

keypath_mutable_impl! {
    bool char String
    usize u8 u16 u32 u64 u128
    isize i8 i16 i32 i64 i128
    f32 f64 DateTime<Utc> Uuid
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use crate::macros::{KeyPathMutable, Navigable};
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{key_path_mutable::KeyPathError, keypath, Change, KeyPath, Navigable};

    #[test]
    fn updates_a_vector_element() {
        let mut data = vec![1, 2, 3];
        let change = Change::update(keypath![Vec<usize>: [1]], 5);

        data.apply_change(&change);

        assert_eq!(data, vec![1, 5, 3]);
    }

    #[test]
    fn splices_a_vector() {
        let mut data = vec![1, 2, 3];
        let change = Change::splice(KeyPath::unit(), vec![5, 6], 1, 0);

        data.apply_change(&change);

        assert_eq!(data, vec![1, 5, 6, 2, 3]);
    }

    #[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Navigable)]
    struct SimpleStruct {
        first_field: usize,
        second_field: String,
        third_field: Vec<String>,
    }

    // This impl will be generated by a derive macro
    impl KeyPathMutable for SimpleStruct {
        fn patch_keypath(
            &mut self,
            keys: &[KeyPathElement],
            patch: Patch,
        ) -> Result<(), KeyPathError> {
            if keys.is_empty() {
                return if let Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(KeyPathError::from_deserialization_error::<SimpleStruct>)?;
                    Ok(())
                } else {
                    Err(KeyPathError::cannot_splice_type::<SimpleStruct>())
                };
            }

            let KeyPathElement::Field { key } = keys[0] else {
                return Err(KeyPathError::must_mutate_struct_with_field::<SimpleStruct>());
            };

            match key {
                // The match arms will be generated by the macro based on the struct fields
                "first_field" => self.first_field.patch_keypath(&keys[1..], patch),
                "different_field" => self.second_field.patch_keypath(&keys[1..], patch),
                "third_field" => self.third_field.patch_keypath(&keys[1..], patch),
                _ => Err(KeyPathError::unknown_field::<SimpleStruct>(key)),
            }
        }
    }

    #[test]
    fn updates_a_struct_field() {
        let mut data = SimpleStruct {
            first_field: 1,
            second_field: "hello".to_string(),
            third_field: vec![],
        };
        let change = Change::update(keypath![SimpleStruct: first_field], 5);

        data.apply_change(&change);

        assert_eq!(
            data,
            SimpleStruct {
                first_field: 5,
                second_field: "hello".to_string(),
                third_field: vec![],
            }
        );
    }

    #[test]
    fn updates_inside_a_struct_field() {
        let mut data = SimpleStruct {
            first_field: 1,
            second_field: "hello".to_string(),
            third_field: vec!["one".to_string(), "two".to_string()],
        };
        let change = Change::update(keypath![SimpleStruct: third_field[1]], "three".to_string());

        data.apply_change(&change);

        assert_eq!(
            data,
            SimpleStruct {
                first_field: 1,
                second_field: "hello".to_string(),
                third_field: vec!["one".to_string(), "three".to_string()],
            }
        );
    }

    #[derive(PartialEq, Debug, Serialize, Deserialize, Navigable)]
    enum ExhaustingEnum {
        First(usize),
        Second { field: String },
        Third(usize, String),
        Fourth { field1: usize, field2: String },
        Fifth(SimpleStruct),
        Sixth { field: SimpleStruct },
    }

    // This impl will be generated by a derive macro
    impl KeyPathMutable for ExhaustingEnum {
        fn patch_keypath(
            &mut self,
            keys: &[KeyPathElement],
            patch: Patch,
        ) -> Result<(), KeyPathError> {
            if keys.is_empty() {
                return if let Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(KeyPathError::from_deserialization_error::<ExhaustingEnum>)?;
                    Ok(())
                } else {
                    Err(KeyPathError::cannot_splice_type::<ExhaustingEnum>())
                };
            }

            let KeyPathElement::Variant { key: variant, .. } = keys[0] else {
                return Err(KeyPathError::must_mutate_enum_with_variant::<ExhaustingEnum>());
            };

            let KeyPathElement::Field { key: field_name } = keys[1] else {
                return Err(KeyPathError::must_mutate_enum_variant_with_field::<
                    ExhaustingEnum,
                >(variant));
            };

            match self {
                ExhaustingEnum::First(value) if variant == "First" && field_name == "0" => {
                    value.patch_keypath(&keys[2..], patch)
                }
                ExhaustingEnum::Second { field }
                    if variant == "Second" && field_name == "field" =>
                {
                    field.patch_keypath(&keys[2..], patch)
                }
                ExhaustingEnum::Third(value1, value2) if variant == "Third" => match field_name {
                    "0" => value1.patch_keypath(&keys[2..], patch),
                    "1" => value2.patch_keypath(&keys[2..], patch),
                    _ => Err(KeyPathError::unknown_field::<ExhaustingEnum>(field_name)),
                },
                ExhaustingEnum::Fourth { field1, field2 } if variant == "Fourth" => {
                    match field_name {
                        "field1" => field1.patch_keypath(&keys[2..], patch),
                        "field2" => field2.patch_keypath(&keys[2..], patch),
                        _ => Err(KeyPathError::unknown_field::<ExhaustingEnum>(field_name)),
                    }
                }
                ExhaustingEnum::Fifth(value) if variant == "Fifth" && field_name == "0" => {
                    value.patch_keypath(&keys[1..], patch)
                }
                ExhaustingEnum::Sixth { field } if variant == "Sixth" && field_name == "field" => {
                    field.patch_keypath(&keys[1..], patch)
                }
                _ => Err(KeyPathError::unknown_variant_or_field::<ExhaustingEnum>(
                    variant, field_name,
                )),
            }
        }
    }

    #[test]
    fn updates_in_an_enum_variant_tuple() {
        let mut data = ExhaustingEnum::First(1);
        let change = Change::update(keypath![ExhaustingEnum: First.0], 5);

        data.apply_change(&change);

        assert_eq!(data, ExhaustingEnum::First(5));
    }

    #[test]
    fn updates_in_an_enum_variant_struct() {
        let mut data = ExhaustingEnum::Second {
            field: "hello".to_string(),
        };
        let change = Change::update(keypath![ExhaustingEnum: Second.field], "world".to_string());

        data.apply_change(&change);

        assert_eq!(
            data,
            ExhaustingEnum::Second {
                field: "world".to_string()
            }
        );
    }

    #[test]
    fn updates_inside_of_a_some() {
        let mut data = Some(SimpleStruct {
            first_field: 1,
            second_field: "Hello".to_string(),
            third_field: vec![],
        });
        let change = Change::update(keypath![Option<SimpleStruct>: Some.first_field], 2);

        data.apply_change(&change);

        assert_eq!(
            data,
            Some(SimpleStruct {
                first_field: 2,
                second_field: "Hello".to_string(),
                third_field: vec![]
            })
        );
    }

    #[test]
    fn replaces_a_some_with_a_some() {
        let mut data = Some(SimpleStruct {
            first_field: 1,
            second_field: "Hello".to_string(),
            third_field: vec![],
        });

        let new_value = SimpleStruct {
            first_field: 2,
            second_field: "Bye!".to_string(),
            third_field: vec!["What".to_string()],
        };

        let change = Change::update(keypath![Option<SimpleStruct>: Some], new_value.clone());

        data.apply_change(&change);

        assert_eq!(data, Some(new_value));
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Navigable)]
    struct StructWithOption {
        field: Option<usize>,
    }

    // This impl will be generated by a derive macro
    impl KeyPathMutable for StructWithOption {
        fn patch_keypath(
            &mut self,
            keys: &[KeyPathElement],
            patch: Patch,
        ) -> Result<(), KeyPathError> {
            if keys.is_empty() {
                return if let Patch::Update { value, .. } = patch {
                    *self = serde_json::from_value(value)
                        .map_err(KeyPathError::from_deserialization_error::<StructWithOption>)?;
                    Ok(())
                } else {
                    Err(KeyPathError::cannot_splice_type::<StructWithOption>())
                };
            }

            let KeyPathElement::Field { key } = keys[0] else {
                return Err(KeyPathError::must_mutate_struct_with_field::<
                    StructWithOption,
                >());
            };

            match key {
                // The match arms will be generated by the macro based on the struct fields
                "field" => self.field.patch_keypath(&keys[1..], patch),
                _ => Err(KeyPathError::unknown_field::<StructWithOption>(key)),
            }
        }
    }

    #[test]
    fn replaces_a_some_with_a_none() {
        let mut data = StructWithOption { field: Some(3) };

        let change = Change::update(keypath![StructWithOption: field], None);

        data.apply_change(&change);

        assert!(data.field.is_none());
    }

    #[test]
    fn replaces_a_none_with_a_none() {
        let mut data = StructWithOption { field: None };

        let change = Change::update(keypath![StructWithOption: field], None);

        data.apply_change(&change);

        assert!(data.field.is_none());
    }

    #[test]
    fn replaces_a_none_with_a_some() {
        let mut data = StructWithOption { field: None };

        let change = Change::update(keypath![StructWithOption: field], Some(3));

        data.apply_change(&change);

        assert_eq!(data.field, Some(3));
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Navigable, KeyPathMutable)]
    #[serde(rename_all = "camelCase")]
    struct AutoStruct {
        number: f32,
        word: String,
    }

    #[test]
    fn macro_derived_on_struct() {
        let mut data = AutoStruct {
            number: 3.0,
            word: "Hello".to_string(),
        };

        let change = Change::update(keypath![AutoStruct: number], 5.0);
        data.apply_change(&change);

        let change = Change::update(keypath![AutoStruct: word], "Goodbye!".to_string());
        data.apply_change(&change);

        assert_eq!(data.number, 5.0);
        assert_eq!(data.word, "Goodbye!".to_string());
    }

    #[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Navigable, KeyPathMutable)]
    #[serde(rename_all = "camelCase")]
    enum AnotherBigEnum {
        OneTuple(usize),
        OneStruct { text: String },
        TwoTuple(usize, String),
        TwoStruct { number: usize, text: String },
        TupleWithStruct(SimpleStruct),
        StructWithStruct { field: SimpleStruct },
    }

    #[test]
    fn macro_derived_on_enum() {
        let mut data = AnotherBigEnum::OneTuple(1);
        let change = Change::update(keypath![AnotherBigEnum: OneTuple.0], 5);
        data.apply_change(&change);

        assert_eq!(data, AnotherBigEnum::OneTuple(5));

        let mut data = AnotherBigEnum::OneStruct {
            text: "hello".to_string(),
        };
        let change = Change::update(keypath![AnotherBigEnum: OneStruct.text], "bye".to_string());
        data.apply_change(&change);

        assert_eq!(
            data,
            AnotherBigEnum::OneStruct {
                text: "bye".to_string()
            }
        );

        let mut data = AnotherBigEnum::TwoTuple(1, "hi".to_string());
        let change = Change::update(keypath![AnotherBigEnum: TwoTuple.0], 5);
        data.apply_change(&change);

        assert_eq!(data, AnotherBigEnum::TwoTuple(5, "hi".to_string()));

        let mut data = AnotherBigEnum::TwoTuple(1, "hi".to_string());
        let change = Change::update(keypath![AnotherBigEnum: TwoTuple.1], "bye".to_string());
        data.apply_change(&change);

        assert_eq!(data, AnotherBigEnum::TwoTuple(1, "bye".to_string()));

        let mut data = AnotherBigEnum::TwoStruct {
            number: 1,
            text: "hi".to_string(),
        };
        let change = Change::update(keypath![AnotherBigEnum: TwoStruct.number], 5);
        data.apply_change(&change);

        assert_eq!(
            data,
            AnotherBigEnum::TwoStruct {
                number: 5,
                text: "hi".to_string()
            }
        );

        let mut data = AnotherBigEnum::TwoStruct {
            number: 1,
            text: "hi".to_string(),
        };
        let change = Change::update(keypath![AnotherBigEnum: TwoStruct.text], "bye".to_string());
        data.apply_change(&change);

        assert_eq!(
            data,
            AnotherBigEnum::TwoStruct {
                number: 1,
                text: "bye".to_string()
            }
        );

        let mut data = AnotherBigEnum::TupleWithStruct(SimpleStruct {
            first_field: 1,
            second_field: "what".to_string(),
            third_field: vec!["yes?".to_string()],
        });
        let change = Change::update(
            keypath![AnotherBigEnum: TupleWithStruct.0.third_field[0]],
            "no".to_string(),
        );
        data.apply_change(&change);

        let AnotherBigEnum::TupleWithStruct(SimpleStruct { third_field, .. }) = data else {
            panic!("data not modified correctly!");
        };
        assert_eq!(third_field[0], "no".to_string());

        let mut data = AnotherBigEnum::StructWithStruct {
            field: SimpleStruct {
                first_field: 1,
                second_field: "what".to_string(),
                third_field: vec!["yes?".to_string()],
            },
        };
        let change = Change::update(
            keypath![AnotherBigEnum: StructWithStruct.field.third_field[0]],
            "no".to_string(),
        );
        data.apply_change(&change);

        let AnotherBigEnum::StructWithStruct {
            field: SimpleStruct { third_field, .. },
        } = data
        else {
            panic!("data not modified correctly!");
        };
        assert_eq!(third_field[0], "no".to_string());
    }

    // Integration test that handles a complex combination of keypath elements.
    // This catches an edge case that we had with encoding concepts, causing `Change::Update` to
    // fail when the concept variant changed. The edge case was triggered because we previously
    // used a custom `KeyPathMutable` implementation on `Concept` to account for serialization
    // through `CodedConcept`. The custom implementation forgot to handle empty keypaths.
    // Now the macro handles coded enums and this test verifies it works.
    #[test]
    fn update_coded_enum_with_direct_dispatch() {
        #[derive(Clone, Debug, Deserialize, KeyPathMutable, PartialEq, Serialize)]
        #[serde(try_from = "CodedEnum", into = "CodedEnum")]
        #[keypath_mutable(direct_dispatch)]
        enum MyEnum {
            First(First),
            Second(Second),
        }

        #[allow(non_snake_case, dead_code)]
        pub struct MyEnumKeyPathReflection<Root> {
            pub First: (KeyPath<Root, CodedEnum>,),
            pub Second: (KeyPath<Root, CodedEnum>,),
        }

        impl Navigable for MyEnum {
            type Reflection<Root> = MyEnumKeyPathReflection<Root>;
            fn append_to_keypath<Root>(path: &KeyPath<Root, Self>) -> Self::Reflection<Root>
            where
                Root: Sized,
            {
                MyEnumKeyPathReflection {
                    First: (path.appending(&KeyPath::unit()),),
                    Second: (path.appending(&KeyPath::unit()),),
                }
            }
        }

        #[derive(Clone, Debug, Deserialize, KeyPathMutable, PartialEq, Serialize)]
        struct First {
            first: usize,
        }

        #[derive(Clone, Debug, Deserialize, KeyPathMutable, PartialEq, Serialize)]
        struct Second {
            second: String,
        }

        impl TryFrom<CodedEnum> for MyEnum {
            type Error = Infallible;

            fn try_from(value: CodedEnum) -> Result<Self, Self::Error> {
                if value.second.is_empty() {
                    Ok(Self::First(First { first: value.first }))
                } else {
                    Ok(Self::Second(Second {
                        second: value.second,
                    }))
                }
            }
        }

        #[derive(Clone, Debug, Default, Deserialize, KeyPathMutable, PartialEq, Serialize)]
        struct CodedEnum {
            #[serde(default)]
            first: usize,
            #[serde(default)]
            second: String,
        }

        impl From<MyEnum> for CodedEnum {
            fn from(value: MyEnum) -> Self {
                match value {
                    MyEnum::First(First { first }) => Self {
                        first,
                        ..Default::default()
                    },
                    MyEnum::Second(Second { second }) => Self {
                        second,
                        ..Default::default()
                    },
                }
            }
        }

        #[derive(Deserialize, KeyPathMutable, Navigable, Serialize)]
        struct State {
            enums: Vec<MyEnum>,
        }

        let mut state = State {
            enums: vec![MyEnum::First(First { first: 1 })],
        };

        let change = Change::Update {
            key_path: keypath![State: enums[0]],
            value: MyEnum::Second(Second {
                second: "2".to_owned(),
            }),
        };

        state.apply_change(&change.into());

        assert_eq!(
            state.enums[0],
            MyEnum::Second(Second {
                second: "2".to_owned(),
            })
        );
    }
}
