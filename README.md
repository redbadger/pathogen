# pathogen

Type-checked keypaths in Rust.

Pathogen allows you to refer to and mutate items in deeply nested data structures in a typesafe way.

## Getting Started

Pathogen depends on `serde` and `serde_json` and the data you will reference and modify has to
implement `Serializable` and `Deserializable`.

Start by installing the dependencies:

```
cargo add serde serde_json pathogen
```

Then you can use the library like in the example below:

## Example

```rust
use pathogen::macros::{KeyPathMutable, Navigable};
use pathogen::{keypath, Change, KeyPathMutable as _, Navigable as _};
use serde::{Deserialize, Serialize};

// Use derive macros to make data structures navigable and mutable

#[derive(Navigable, KeyPathMutable, Serialize, Deserialize)]
struct Test {
    my_scalar: usize,
    my_vector: Vec<usize>,
    my_nested: Nested,
    my_vector_of_nested: Vec<Nested>,
}

#[derive(Navigable, KeyPathMutable, Serialize, Deserialize)]
struct Nested {
    my_string: String,
    my_vector: Vec<f64>,
}

fn main() {
    let mut test = Test {
        my_scalar: 1,
        my_vector: vec![2, 3, 4],
        my_nested: Nested {
            my_string: "Hello".to_string(),
            my_vector: vec![],
        },
        my_vector_of_nested: vec![],
    };

    // construct keypaths using the keypath! macro with type checking

    let third_number = keypath![Test: my_vector[2]];

    // create changes as values

    let change_to_5 = Change::update(third_number, 5);

    assert_eq!(test.my_vector[2], 4);

    // and apply them to your data structures

    test.apply_change(&change_to_5);

    assert_eq!(test.my_vector[2], 5);

    // you can also update collections with splice

    let append_nested = Change::splice(
        keypath![Test: my_vector_of_nested],
        vec![Nested {
            my_string: "World".to_string(),
            my_vector: vec![1.0, 2.0, 3.0],
        }],
        0,
        0,
    );
    test.apply_change(&append_nested);

    assert_eq!(test.my_vector_of_nested[0].my_string, "World");

    let remove_first_nested = Change::splice(keypath![Test: my_vector_of_nested], vec![], 0, 1);
    test.apply_change(&remove_first_nested);

    assert!(test.my_vector_of_nested.is_empty());
}

```
