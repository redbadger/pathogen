use serde::{Deserialize, Serialize};
use std::{fmt::Display, marker::PhantomData};

use crate::{IndexNavigable, Navigable};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VariantTagType {
    External,
    Internal,
    Adjacent,
    Untagged,
}

/// Path on type Root to a (nested) property of type Value
#[derive(Debug, Serialize, PartialEq)]
#[serde(transparent)]
pub struct KeyPath<Root, Value> {
    pub path: Vec<KeyPathElement>,
    #[serde(skip)]
    root: PhantomData<Root>,
    #[serde(skip)]
    value: PhantomData<Value>,
}

// Implement clone manualy in order to not require `Root` and `Value` to also be Clone
impl<Root, Value> Clone for KeyPath<Root, Value> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            root: self.root,
            value: self.value,
        }
    }
}

/// A KeyPath element, either a field, an enum variant or an index
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum KeyPathElement {
    /// A struct field
    Field { key: &'static str },
    /// An enum variant - note that if the enum instance turns out to be a different variant
    /// the rest of the keypath is invalid. In other words the type checking only makes sure the
    /// keypath is plausible, not that it is actually valid.
    Variant {
        key: &'static str,
        tag: VariantTagType,
    },
    /// A vector index
    Index { key: usize },
    /// A String key in a HashMap or BTReeMap
    StringKey { key: String },
}

impl Display for KeyPathElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyPathElement::Field { key } => write!(f, "{}", key),
            KeyPathElement::Variant { key, .. } => write!(f, "{}", key),
            KeyPathElement::Index { key } => write!(f, "[{}]", key),
            KeyPathElement::StringKey { key } => write!(f, "[\"{}\"]", key),
        }
    }
}

impl<Root, Value> KeyPath<Root, Value> {
    /// Construct a keypath pointing to a struct field
    pub fn field(name: &'static str) -> Self {
        Self {
            path: vec![KeyPathElement::Field { key: name }],
            root: PhantomData::<Root>,
            value: PhantomData::<Value>,
        }
    }

    /// Construct a keypath pointing to an enum variant
    pub fn variant(key: &'static str, tag: VariantTagType) -> Self {
        Self {
            path: vec![KeyPathElement::Variant { key, tag }],
            root: PhantomData::<Root>,
            value: PhantomData::<Value>,
        }
    }

    /// Construct a keypath pointing to a tuple variant field
    pub fn tuple_variant(key: &'static str, index: &'static str, tag: VariantTagType) -> Self {
        Self {
            path: vec![
                KeyPathElement::Variant { key, tag },
                KeyPathElement::Field { key: index },
            ],
            root: PhantomData::<Root>,
            value: PhantomData::<Value>,
        }
    }

    /// Construct a keypath pointing to an index
    pub fn index(index: usize) -> Self {
        Self {
            path: vec![KeyPathElement::Index { key: index }],
            root: PhantomData::<Root>,
            value: PhantomData::<Value>,
        }
    }

    pub fn string_key<K: Display>(key: K) -> Self {
        Self {
            path: vec![KeyPathElement::StringKey {
                key: format!("{key}"),
            }],
            root: PhantomData::<Root>,
            value: PhantomData::<Value>,
        }
    }

    /// Construct an empty 'unit' keypath
    pub fn unit() -> Self {
        KeyPath {
            path: vec![],
            root: PhantomData::<Root>,
            value: PhantomData::<Value>,
        }
    }

    /// Append another keypath to this one
    pub fn appending<T>(&self, next: &KeyPath<Value, T>) -> KeyPath<Root, T> {
        let mut path = self.path.clone();
        path.extend(next.path.clone());

        KeyPath {
            path,
            root: PhantomData::<Root>,
            value: PhantomData::<T>,
        }
    }

    /// Append this keypath to another one
    fn prepending<T>(&self, previous: &KeyPath<T, Root>) -> KeyPath<T, Value> {
        previous.appending(self)
    }

    /// Unsafely construct a keypath with pre-constructed path elements
    /// This is 'dangerous' because we cannot statically guarantee that following the path
    /// from a value of type Root will result in a value of type Value
    pub fn dangerously_construct_from_path(path: Vec<KeyPathElement>) -> Self {
        Self {
            path,
            root: PhantomData::<Root>,
            value: PhantomData::<Value>,
        }
    }

    // Fluent API

    /// Get all paths to fields which can be navigated from this keypath
    pub fn fields(&self) -> Value::Reflection<Root>
    where
        Value: Navigable,
    {
        Value::append_to_keypath(self)
    }

    /// Append a vector index to this keypath
    pub fn at<K, V>(&self, index: K) -> KeyPath<Root, V>
    where
        Value: IndexNavigable<K, V>,
    {
        Value::index_keypath_segment(index).prepending(self)
    }
}

/// Partially erased keypath, retaining information about the root type, but erasing the value type
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct KeyPathFrom<Root> {
    pub path: Vec<KeyPathElement>,
    root: PhantomData<Root>,
}

impl<T> Display for KeyPathFrom<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ".")?;
        for (ix, p) in self.path.iter().enumerate() {
            write!(f, "{}", p)?;
            if ix + 1 != self.path.len()
                && matches!(
                    p,
                    KeyPathElement::Field { .. } | KeyPathElement::Variant { .. }
                )
            {
                write!(f, ".")?;
            }
        }
        Ok(())
    }
}

impl<Root> KeyPathFrom<Root> {
    pub fn prepending<Base>(&self, keypath: &KeyPath<Base, Root>) -> KeyPathFrom<Base> {
        let mut path = keypath.path.clone();
        path.extend(self.path.clone());

        KeyPathFrom {
            path,
            root: PhantomData::<Base>,
        }
    }

    /// Returns whether this subpath is fully contained within `other`.
    ///
    /// In other words, whether `other` references a field/index/variant within
    /// `self`.
    ///
    /// Returns `false` if both paths are equal.
    pub fn is_subpath_of(&self, other: &Self) -> bool {
        if other.path.len() <= self.path.len() {
            return false;
        }

        for (own_element, other_element) in self.path.iter().zip(other.path.iter()) {
            if own_element != other_element {
                return false;
            }
        }

        true
    }

    /// Downcast this keypath to include value type. Note that this always succeeds, regardless of the actual value type
    /// the path is pointing to, use with caution.
    pub fn downcast<T>(&self) -> KeyPath<Root, T> {
        KeyPath {
            path: self.path.clone(),
            root: PhantomData::<Root>,
            value: PhantomData::<T>,
        }
    }
}

impl<Root, T> From<KeyPath<Root, T>> for KeyPathFrom<Root> {
    fn from(value: KeyPath<Root, T>) -> Self {
        KeyPathFrom {
            path: value.path,
            root: PhantomData::<Root>,
        }
    }
}

impl<Root, T> PartialEq<KeyPath<Root, T>> for KeyPathFrom<Root> {
    fn eq(&self, other: &KeyPath<Root, T>) -> bool {
        self.path == other.path
    }
}
