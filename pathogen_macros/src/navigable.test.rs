use darling::FromDeriveInput;
use quote::quote;
use syn::parse_str;

use super::NavigableType;

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
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                a: path.appending(&key_path::KeyPath::field("a")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub a: key_path::KeyPath<Root, usize>,
    }
    "###);
}

#[test]
fn struct_with_multiple_fields() {
    let input = r#"
            #[derive(Navigable)]
            struct MyStruct {
                my_string: String,
                my_vector: Vec<usize>,
                my_structs: Vec<Nested>,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                my_string: path.appending(&key_path::KeyPath::field("my_string")),
                my_vector: path.appending(&key_path::KeyPath::field("my_vector")),
                my_structs: path.appending(&key_path::KeyPath::field("my_structs")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub my_string: key_path::KeyPath<Root, String>,
        pub my_vector: key_path::KeyPath<Root, Vec<usize>>,
        pub my_structs: key_path::KeyPath<Root, Vec<Nested>>,
    }
    "###);
}

#[test]
fn enum_with_struct_variants() {
    let input = r#"
            #[derive(Navigable)]
            enum MyEnum {
                FirstOne { a: usize },
                SecondOne { b: String, c: f64 },
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    pub struct MyEnumKeyPathReflectionVariantFirstOne<Root> {
        pub a: key_path::KeyPath<Root, usize>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&key_path::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: key_path::KeyPath<Root, String>,
        pub c: key_path::KeyPath<Root, f64>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&key_path::KeyPath::field("b")),
                c: path.appending(&key_path::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: key_path::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: key_path::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl key_path::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "FirstOne",
                            key_path::VariantTagType::External,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "SecondOne",
                            key_path::VariantTagType::External,
                        ),
                    ),
            }
        }
    }
    "###);
}

#[test]
fn enum_with_tuple_variants() {
    let input = r#"
            #[derive(Navigable)]
            enum TestTupleEnum {
                VariantOne(usize),
                VariantTwo(Nested, String),
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    #[allow(non_snake_case)]
    pub struct TestTupleEnumKeyPathReflection<Root> {
        pub VariantOne: (key_path::KeyPath<Root, usize>,),
        pub VariantTwo: (key_path::KeyPath<Root, Nested>, key_path::KeyPath<Root, String>),
    }
    impl key_path::Navigable for TestTupleEnum {
        type Reflection<Root> = TestTupleEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            TestTupleEnumKeyPathReflection {
                VariantOne: (
                    path
                        .appending(
                            &key_path::KeyPath::tuple_variant(
                                "VariantOne",
                                "0",
                                key_path::VariantTagType::External,
                            ),
                        ),
                ),
                VariantTwo: (
                    path
                        .appending(
                            &key_path::KeyPath::tuple_variant(
                                "VariantTwo",
                                "0",
                                key_path::VariantTagType::External,
                            ),
                        ),
                    path
                        .appending(
                            &key_path::KeyPath::tuple_variant(
                                "VariantTwo",
                                "1",
                                key_path::VariantTagType::External,
                            ),
                        ),
                ),
            }
        }
    }
    "###);
}

#[test]
fn struct_with_serde_rename() {
    let input = r#"
            #[derive(Navigable)]
            struct MyStruct {
                #[serde(rename = "bob")]
                my_string: String,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                my_string: path.appending(&key_path::KeyPath::field("bob")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub my_string: key_path::KeyPath<Root, String>,
    }
    "###);
}

#[test]
fn struct_with_serde_rename_and_default() {
    let input = r#"
            #[derive(Navigable)]
            struct MyStruct {
                #[serde(default, rename = "bob")]
                my_string: String,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                my_string: path.appending(&key_path::KeyPath::field("bob")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub my_string: key_path::KeyPath<Root, String>,
    }
    "###);
}

#[test]
fn enum_with_serde_rename() {
    let input = r#"
            #[derive(Navigable)]
            enum MyEnum {
                #[serde(rename = "first")]
                FirstOne { a: usize },
                #[serde(rename = "second")]
                SecondOne { b: String, c: f64 },
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    pub struct MyEnumKeyPathReflectionVariantFirstOne<Root> {
        pub a: key_path::KeyPath<Root, usize>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&key_path::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: key_path::KeyPath<Root, String>,
        pub c: key_path::KeyPath<Root, f64>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&key_path::KeyPath::field("b")),
                c: path.appending(&key_path::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: key_path::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: key_path::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl key_path::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "first",
                            key_path::VariantTagType::External,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "second",
                            key_path::VariantTagType::External,
                        ),
                    ),
            }
        }
    }
    "###);
}
#[test]
fn struct_with_serde_rename_all() {
    let input = r#"
            #[derive(Navigable)]
            #[serde(rename_all = "camelCase")]
            struct MyStruct {
                my_string: String,
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    impl key_path::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                my_string: path.appending(&key_path::KeyPath::field("myString")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub my_string: key_path::KeyPath<Root, String>,
    }
    "###);
}

#[test]
fn enum_with_serde_rename_all() {
    let input = r#"
            #[derive(Navigable)]
            #[serde(rename_all = "camelCase")]
            enum MyEnum {
                FirstOne { a: usize },
                SecondOne { b: String, c: f64 },
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    pub struct MyEnumKeyPathReflectionVariantFirstOne<Root> {
        pub a: key_path::KeyPath<Root, usize>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&key_path::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: key_path::KeyPath<Root, String>,
        pub c: key_path::KeyPath<Root, f64>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&key_path::KeyPath::field("b")),
                c: path.appending(&key_path::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: key_path::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: key_path::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl key_path::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "firstOne",
                            key_path::VariantTagType::External,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "secondOne",
                            key_path::VariantTagType::External,
                        ),
                    ),
            }
        }
    }
    "###);
}

#[test]
fn externally_tagged_enum() {
    let input = r#"
            #[derive(Navigable)]
            enum MyEnum {
                FirstOne { a: usize },
                SecondOne { b: String, c: f64 },
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    pub struct MyEnumKeyPathReflectionVariantFirstOne<Root> {
        pub a: key_path::KeyPath<Root, usize>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&key_path::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: key_path::KeyPath<Root, String>,
        pub c: key_path::KeyPath<Root, f64>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&key_path::KeyPath::field("b")),
                c: path.appending(&key_path::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: key_path::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: key_path::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl key_path::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "FirstOne",
                            key_path::VariantTagType::External,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "SecondOne",
                            key_path::VariantTagType::External,
                        ),
                    ),
            }
        }
    }
    "###);
}

#[test]
fn internally_tagged_enum() {
    let input = r#"
            #[derive(Navigable)]
            #[serde(tag = "type")]
            enum MyEnum {
                FirstOne { a: usize },
                SecondOne { b: String, c: f64 },
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    pub struct MyEnumKeyPathReflectionVariantFirstOne<Root> {
        pub a: key_path::KeyPath<Root, usize>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&key_path::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: key_path::KeyPath<Root, String>,
        pub c: key_path::KeyPath<Root, f64>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&key_path::KeyPath::field("b")),
                c: path.appending(&key_path::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: key_path::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: key_path::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl key_path::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "FirstOne",
                            key_path::VariantTagType::Internal,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "SecondOne",
                            key_path::VariantTagType::Internal,
                        ),
                    ),
            }
        }
    }
    "###);
}

#[test]
fn adjacently_tagged_enum() {
    let input = r#"
            #[derive(Navigable)]
            #[serde(tag = "type", content = "data")]
            enum MyEnum {
                FirstOne { a: usize },
                SecondOne { b: String, c: f64 },
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    pub struct MyEnumKeyPathReflectionVariantFirstOne<Root> {
        pub a: key_path::KeyPath<Root, usize>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&key_path::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: key_path::KeyPath<Root, String>,
        pub c: key_path::KeyPath<Root, f64>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&key_path::KeyPath::field("b")),
                c: path.appending(&key_path::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: key_path::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: key_path::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl key_path::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "FirstOne",
                            key_path::VariantTagType::Adjacent,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "SecondOne",
                            key_path::VariantTagType::Adjacent,
                        ),
                    ),
            }
        }
    }
    "###);
}

#[test]
fn untagged_enum() {
    let input = r#"
            #[derive(Navigable)]
            #[serde(untagged)]
            enum MyEnum {
                FirstOne { a: usize },
                SecondOne { b: String, c: f64 },
            }
        "#;

    let input = parse_str(input).unwrap();
    let input = NavigableType::from_derive_input(&input).unwrap();

    let actual = quote!(#input);

    insta::assert_snapshot!(pretty_print(&actual), @r###"
    pub struct MyEnumKeyPathReflectionVariantFirstOne<Root> {
        pub a: key_path::KeyPath<Root, usize>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&key_path::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: key_path::KeyPath<Root, String>,
        pub c: key_path::KeyPath<Root, f64>,
    }
    impl<T> key_path::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&key_path::KeyPath::field("b")),
                c: path.appending(&key_path::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: key_path::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: key_path::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl key_path::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &key_path::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "FirstOne",
                            key_path::VariantTagType::Untagged,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &key_path::KeyPath::variant(
                            "SecondOne",
                            key_path::VariantTagType::Untagged,
                        ),
                    ),
            }
        }
    }
    "###);
}
