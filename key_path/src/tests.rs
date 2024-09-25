use std::collections::{BTreeMap, HashMap};

use pretty_assertions::assert_eq;
use serde::Serialize;
use serde_json::json;

use super::*;
use crate::{macros::Navigable, navigable::Navigable};

#[derive(Navigable)]
#[allow(dead_code)] // Only reflection is tested
struct Test {
    my_scalar: usize,
    my_vector: Vec<usize>,
    my_nested: Nested,
    my_vector_of_nested: Vec<Nested>,
}

#[derive(Navigable)]
#[allow(dead_code)] // Only reflection is tested
struct Nested {
    my_string: String,
    my_vector: Vec<f64>,
}

#[test]
fn one_step_keypath() {
    let keypath: KeyPath<Test, usize> = Test::keypaths().my_scalar;

    assert_eq!(
        keypath.path,
        vec![KeyPathElement::Field { key: "my_scalar" }]
    );
}

#[test]
fn two_step_keypath() {
    let keypath: KeyPath<Test, String> = Test::keypaths().my_nested.fields().my_string;

    assert_eq!(
        keypath.path,
        vec![
            KeyPathElement::Field { key: "my_nested" },
            KeyPathElement::Field { key: "my_string" }
        ]
    );
}

#[test]
fn two_step_keypath_with_index() {
    let keypath: KeyPath<Test, usize> = Test::keypaths().my_vector.at(0);

    assert_eq!(
        keypath.path,
        vec![
            KeyPathElement::Field { key: "my_vector" },
            KeyPathElement::Index { key: 0 }
        ]
    );
}

#[test]
fn deeper_keypath() {
    let keypath: KeyPath<Test, String> = Test::keypaths()
        .my_vector_of_nested
        .at(0)
        .fields()
        .my_string;

    assert_eq!(
        keypath.path,
        vec![
            KeyPathElement::Field {
                key: "my_vector_of_nested"
            },
            KeyPathElement::Index { key: 0 },
            KeyPathElement::Field { key: "my_string" }
        ]
    );
}

#[test]
fn keypath_with_multiple_vectors() {
    let keypath: KeyPath<Test, f64> = Test::keypaths()
        .my_vector_of_nested
        .at(0)
        .fields()
        .my_vector
        .at(0);

    assert_eq!(
        keypath.path,
        vec![
            KeyPathElement::Field {
                key: "my_vector_of_nested"
            },
            KeyPathElement::Index { key: 0 },
            KeyPathElement::Field { key: "my_vector" },
            KeyPathElement::Index { key: 0 }
        ]
    );
}

#[test]
fn keypath_macro() {
    let keypath: KeyPath<Test, f64> = keypath![Test: my_vector_of_nested[0].my_vector[0]];

    assert_eq!(
        keypath.path,
        vec![
            KeyPathElement::Field {
                key: "my_vector_of_nested"
            },
            KeyPathElement::Index { key: 0 },
            KeyPathElement::Field { key: "my_vector" },
            KeyPathElement::Index { key: 0 }
        ]
    );
}

#[test]
fn keypath_macro_on_vector() {
    let keypath: KeyPath<Vec<Nested>, f64> = keypath![Vec::<Nested>: [0].my_vector[0]];

    assert_eq!(
        keypath.path,
        vec![
            KeyPathElement::Index { key: 0 },
            KeyPathElement::Field { key: "my_vector" },
            KeyPathElement::Index { key: 0 }
        ]
    );
}

#[test]
fn keypath_macro_on_vector_dyn() {
    let index = 5;
    let keypath: KeyPath<Vec<Nested>, f64> = keypath![Vec::<Nested>: [index].my_vector[0]];

    assert_eq!(
        keypath.path,
        vec![
            KeyPathElement::Index { key: 5 },
            KeyPathElement::Field { key: "my_vector" },
            KeyPathElement::Index { key: 0 }
        ]
    );
}

#[derive(Navigable)]
#[allow(dead_code)] // Only reflection is tested
enum EnumTest {
    TestVariant { test: Test },
    NestedVariant { nested: Nested },
}

#[derive(Navigable)]
#[allow(dead_code)] // Only reflection is tested
struct StructWithEnum {
    my_enum: EnumTest,
}

#[test]
fn enum_keypaths() {
    let test_keypath: KeyPath<EnumTest, usize> = keypath![EnumTest: TestVariant.test.my_scalar];
    let nested_keypath: KeyPath<EnumTest, String> =
        keypath![EnumTest: NestedVariant.nested.my_string];

    assert_eq!(
        test_keypath.path,
        vec![
            KeyPathElement::Variant {
                key: "TestVariant",
                tag: VariantTagType::External
            },
            KeyPathElement::Field { key: "test" },
            KeyPathElement::Field { key: "my_scalar" }
        ]
    );
    assert_eq!(
        nested_keypath.path,
        vec![
            KeyPathElement::Variant {
                key: "NestedVariant",
                tag: VariantTagType::External
            },
            KeyPathElement::Field { key: "nested" },
            KeyPathElement::Field { key: "my_string" }
        ]
    );
}

#[test]
fn nested_enum_keypaths() {
    let test_keypath: KeyPath<StructWithEnum, usize> =
        keypath![StructWithEnum: my_enum.TestVariant.test.my_scalar];
    let nested_keypath: KeyPath<StructWithEnum, String> =
        keypath![StructWithEnum: my_enum.NestedVariant.nested.my_string];

    assert_eq!(
        test_keypath.path,
        vec![
            KeyPathElement::Field { key: "my_enum" },
            KeyPathElement::Variant {
                key: "TestVariant",
                tag: VariantTagType::External
            },
            KeyPathElement::Field { key: "test" },
            KeyPathElement::Field { key: "my_scalar" }
        ]
    );
    assert_eq!(
        nested_keypath.path,
        vec![
            KeyPathElement::Field { key: "my_enum" },
            KeyPathElement::Variant {
                key: "NestedVariant",
                tag: VariantTagType::External
            },
            KeyPathElement::Field { key: "nested" },
            KeyPathElement::Field { key: "my_string" }
        ]
    );
}

#[test]
fn basic_serialization() {
    let keypath: KeyPath<Test, usize> = keypath![Test: my_scalar];
    let serialized = serde_json::to_string(&keypath).unwrap();

    assert_eq!(serialized, r#"[{"type":"field","key":"my_scalar"}]"#);
}

#[test]
fn complex_serialization() {
    let keypath: KeyPath<StructWithEnum, f64> =
        keypath![StructWithEnum: my_enum.TestVariant.test.my_vector_of_nested[4].my_vector[0]];
    let serialized = serde_json::to_value(keypath).unwrap();

    assert_eq!(
        serialized,
        json! {
            [
                {"type":"field","key":"my_enum"},
                {"type":"variant","key":"TestVariant","tag":"external"},
                {"type":"field","key":"test"},
                {"type":"field","key":"my_vector_of_nested"},
                {"type":"index","key":4},
                {"type":"field","key":"my_vector"},
                {"type":"index","key":0}
            ]
        }
    );
}

#[derive(Serialize, Navigable)]
#[serde(rename_all = "camelCase")]
struct RenamedStruct {
    my_field: usize,
}

#[allow(dead_code)] // Only reflection is tested
#[derive(Serialize, Navigable)]
#[serde(rename_all = "camelCase")]
enum RenamedEnum {
    #[serde(rename_all = "camelCase")]
    VariantOne {
        my_field: usize,
    },
    // intentionally no rename
    VariantTwo {
        my_field: RenamedStruct,
    },
}

#[test]
fn serialization_respects_rename_all() {
    let keypath: KeyPath<RenamedEnum, usize> = keypath![RenamedEnum: VariantOne.my_field];
    let serialized = serde_json::to_value(keypath).unwrap();

    assert_eq!(
        serialized,
        json! {
            [
                {"type":"variant","key":"variantOne","tag":"external"},
                {"type":"field","key":"myField"}
            ]
        }
    );

    let keypath: KeyPath<RenamedEnum, usize> = keypath![RenamedEnum: VariantTwo.my_field.my_field];
    let serialized = serde_json::to_value(keypath).unwrap();

    assert_eq!(
        serialized,
        json! {
            [
                {"type":"variant","key":"variantTwo","tag":"external"},
                {"type":"field","key":"my_field"},
                {"type":"field","key":"myField"}
            ]
        }
    );
}

#[allow(dead_code)] // Only reflection is tested
#[derive(Navigable)]
enum TestTupleEnum {
    VariantOne(usize),
    VariantTwo(Nested, String),
}

#[test]
fn enum_keypaths_tuple_variants() {
    let one_keypath: KeyPath<TestTupleEnum, usize> = keypath![TestTupleEnum: VariantOne.0];
    let two_keypath: KeyPath<TestTupleEnum, String> = keypath![TestTupleEnum: VariantTwo.1];
    let two_keypath_deep: KeyPath<TestTupleEnum, f64> =
        keypath![TestTupleEnum: VariantTwo.0.my_vector[0]];

    assert_eq!(
        one_keypath.path,
        vec![
            KeyPathElement::Variant {
                key: "VariantOne",
                tag: VariantTagType::External
            },
            KeyPathElement::Field { key: "0" },
        ]
    );

    assert_eq!(
        two_keypath.path,
        vec![
            KeyPathElement::Variant {
                key: "VariantTwo",
                tag: VariantTagType::External
            },
            KeyPathElement::Field { key: "1" },
        ]
    );

    assert_eq!(
        two_keypath_deep.path,
        vec![
            KeyPathElement::Variant {
                key: "VariantTwo",
                tag: VariantTagType::External
            },
            KeyPathElement::Field { key: "0" },
            KeyPathElement::Field { key: "my_vector" },
            KeyPathElement::Index { key: 0 },
        ]
    );
}

#[derive(Navigable)]
#[allow(dead_code)]
struct ThingWithMaps {
    string_counts: HashMap<&'static str, usize>,
    sparse_strings: BTreeMap<usize, String>,
}

#[test]
fn keypath_into_maps() {
    let string_count = keypath![ThingWithMaps: string_counts["Hello"]];
    let sparse_string = keypath![ThingWithMaps: sparse_strings[3]];

    assert_eq!(
        string_count.path,
        vec![
            KeyPathElement::Field {
                key: "string_counts"
            },
            KeyPathElement::StringKey {
                key: "Hello".to_string()
            }
        ]
    );

    assert_eq!(
        sparse_string.path,
        vec![
            KeyPathElement::Field {
                key: "sparse_strings"
            },
            KeyPathElement::StringKey { key: 3.to_string() }
        ]
    );
}
