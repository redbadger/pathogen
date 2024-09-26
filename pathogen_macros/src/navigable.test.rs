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
    impl pathogen::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                a: path.appending(&pathogen::KeyPath::field("a")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub a: pathogen::KeyPath<Root, usize>,
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
    impl pathogen::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                my_string: path.appending(&pathogen::KeyPath::field("my_string")),
                my_vector: path.appending(&pathogen::KeyPath::field("my_vector")),
                my_structs: path.appending(&pathogen::KeyPath::field("my_structs")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub my_string: pathogen::KeyPath<Root, String>,
        pub my_vector: pathogen::KeyPath<Root, Vec<usize>>,
        pub my_structs: pathogen::KeyPath<Root, Vec<Nested>>,
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
        pub a: pathogen::KeyPath<Root, usize>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&pathogen::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: pathogen::KeyPath<Root, String>,
        pub c: pathogen::KeyPath<Root, f64>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&pathogen::KeyPath::field("b")),
                c: path.appending(&pathogen::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: pathogen::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: pathogen::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl pathogen::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "FirstOne",
                            pathogen::VariantTagType::External,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "SecondOne",
                            pathogen::VariantTagType::External,
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
        pub VariantOne: (pathogen::KeyPath<Root, usize>,),
        pub VariantTwo: (pathogen::KeyPath<Root, Nested>, pathogen::KeyPath<Root, String>),
    }
    impl pathogen::Navigable for TestTupleEnum {
        type Reflection<Root> = TestTupleEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            TestTupleEnumKeyPathReflection {
                VariantOne: (
                    path
                        .appending(
                            &pathogen::KeyPath::tuple_variant(
                                "VariantOne",
                                "0",
                                pathogen::VariantTagType::External,
                            ),
                        ),
                ),
                VariantTwo: (
                    path
                        .appending(
                            &pathogen::KeyPath::tuple_variant(
                                "VariantTwo",
                                "0",
                                pathogen::VariantTagType::External,
                            ),
                        ),
                    path
                        .appending(
                            &pathogen::KeyPath::tuple_variant(
                                "VariantTwo",
                                "1",
                                pathogen::VariantTagType::External,
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
    impl pathogen::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                my_string: path.appending(&pathogen::KeyPath::field("bob")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub my_string: pathogen::KeyPath<Root, String>,
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
    impl pathogen::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                my_string: path.appending(&pathogen::KeyPath::field("bob")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub my_string: pathogen::KeyPath<Root, String>,
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
        pub a: pathogen::KeyPath<Root, usize>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&pathogen::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: pathogen::KeyPath<Root, String>,
        pub c: pathogen::KeyPath<Root, f64>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&pathogen::KeyPath::field("b")),
                c: path.appending(&pathogen::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: pathogen::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: pathogen::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl pathogen::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "first",
                            pathogen::VariantTagType::External,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "second",
                            pathogen::VariantTagType::External,
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
    impl pathogen::Navigable for MyStruct {
        type Reflection<Root> = MyStructKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyStructKeyPathReflection {
                my_string: path.appending(&pathogen::KeyPath::field("myString")),
            }
        }
    }
    pub struct MyStructKeyPathReflection<Root> {
        pub my_string: pathogen::KeyPath<Root, String>,
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
        pub a: pathogen::KeyPath<Root, usize>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&pathogen::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: pathogen::KeyPath<Root, String>,
        pub c: pathogen::KeyPath<Root, f64>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&pathogen::KeyPath::field("b")),
                c: path.appending(&pathogen::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: pathogen::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: pathogen::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl pathogen::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "firstOne",
                            pathogen::VariantTagType::External,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "secondOne",
                            pathogen::VariantTagType::External,
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
        pub a: pathogen::KeyPath<Root, usize>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&pathogen::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: pathogen::KeyPath<Root, String>,
        pub c: pathogen::KeyPath<Root, f64>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&pathogen::KeyPath::field("b")),
                c: path.appending(&pathogen::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: pathogen::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: pathogen::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl pathogen::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "FirstOne",
                            pathogen::VariantTagType::External,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "SecondOne",
                            pathogen::VariantTagType::External,
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
        pub a: pathogen::KeyPath<Root, usize>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&pathogen::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: pathogen::KeyPath<Root, String>,
        pub c: pathogen::KeyPath<Root, f64>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&pathogen::KeyPath::field("b")),
                c: path.appending(&pathogen::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: pathogen::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: pathogen::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl pathogen::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "FirstOne",
                            pathogen::VariantTagType::Internal,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "SecondOne",
                            pathogen::VariantTagType::Internal,
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
        pub a: pathogen::KeyPath<Root, usize>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&pathogen::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: pathogen::KeyPath<Root, String>,
        pub c: pathogen::KeyPath<Root, f64>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&pathogen::KeyPath::field("b")),
                c: path.appending(&pathogen::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: pathogen::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: pathogen::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl pathogen::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "FirstOne",
                            pathogen::VariantTagType::Adjacent,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "SecondOne",
                            pathogen::VariantTagType::Adjacent,
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
        pub a: pathogen::KeyPath<Root, usize>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantFirstOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantFirstOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantFirstOne {
                a: path.appending(&pathogen::KeyPath::field("a")),
            }
        }
    }
    pub struct MyEnumKeyPathReflectionVariantSecondOne<Root> {
        pub b: pathogen::KeyPath<Root, String>,
        pub c: pathogen::KeyPath<Root, f64>,
    }
    impl<T> pathogen::Navigable for MyEnumKeyPathReflectionVariantSecondOne<T> {
        type Reflection<Root> = MyEnumKeyPathReflectionVariantSecondOne<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflectionVariantSecondOne {
                b: path.appending(&pathogen::KeyPath::field("b")),
                c: path.appending(&pathogen::KeyPath::field("c")),
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct MyEnumKeyPathReflection<Root> {
        pub FirstOne: pathogen::KeyPath<Root, MyEnumKeyPathReflectionVariantFirstOne<Root>>,
        pub SecondOne: pathogen::KeyPath<
            Root,
            MyEnumKeyPathReflectionVariantSecondOne<Root>,
        >,
    }
    impl pathogen::Navigable for MyEnum {
        type Reflection<Root> = MyEnumKeyPathReflection<Root>;
        fn append_to_keypath<Root>(
            path: &pathogen::KeyPath<Root, Self>,
        ) -> Self::Reflection<Root>
        where
            Root: Sized,
        {
            MyEnumKeyPathReflection {
                FirstOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "FirstOne",
                            pathogen::VariantTagType::Untagged,
                        ),
                    ),
                SecondOne: path
                    .appending(
                        &pathogen::KeyPath::variant(
                            "SecondOne",
                            pathogen::VariantTagType::Untagged,
                        ),
                    ),
            }
        }
    }
    "###);
}
