use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
};

use crate::KeyPath;

/// Trait for types that can be navigated with key paths
pub trait Navigable
where
    Self: Sized,
{
    /// Paths from Self to properties on Self
    type Reflection<T>;

    fn keypaths() -> Self::Reflection<Self> {
        Self::append_to_keypath(&KeyPath::<Self, Self>::unit())
    }

    fn append_to_keypath<R>(path: &KeyPath<R, Self>) -> Self::Reflection<R>
    where
        R: Sized;
}

/// Trait for types that can be indexed with key paths
pub trait IndexNavigable<K, V>
where
    Self: Sized,
{
    fn index_keypath_segment(index: K) -> KeyPath<Self, V>;
}

impl<T> IndexNavigable<usize, T> for Vec<T> {
    fn index_keypath_segment(index: usize) -> KeyPath<Vec<T>, T> {
        KeyPath::index(index)
    }
}

impl<K: Display, V> IndexNavigable<K, V> for HashMap<K, V> {
    fn index_keypath_segment(index: K) -> KeyPath<Self, V> {
        KeyPath::string_key(format!("{index}"))
    }
}

impl<K: Display, V> IndexNavigable<K, V> for BTreeMap<K, V> {
    fn index_keypath_segment(index: K) -> KeyPath<Self, V> {
        KeyPath::string_key(format!("{index}"))
    }
}

impl<T: Navigable> Navigable for Option<T> {
    type Reflection<Root> = SomeReflection<Root, T>;

    fn append_to_keypath<R>(path: &KeyPath<R, Self>) -> Self::Reflection<R>
    where
        R: Sized,
    {
        SomeReflection {
            Some: path.appending(&KeyPath::unit()),
        }
    }
}

#[allow(non_snake_case)]
pub struct SomeReflection<Root, T: Navigable> {
    pub Some: KeyPath<Root, T>,
}

impl<PreviousRoot, T: Navigable> Navigable for SomeReflection<PreviousRoot, T> {
    type Reflection<Root> = T::Reflection<Root>;

    fn append_to_keypath<R>(path: &KeyPath<R, Self>) -> Self::Reflection<R>
    where
        R: Sized,
    {
        T::append_to_keypath(&path.appending(&KeyPath::unit()))
    }
}
