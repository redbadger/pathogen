pub mod key_path;
pub mod key_path_mutable;

mod keypath_macro;
mod navigable;

pub mod macros {
    pub use key_path_macros::{KeyPathMutable, Navigable};
}

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub use key_path::{KeyPath, KeyPathElement, KeyPathFrom, VariantTagType};
pub use key_path_mutable::{KeyPathError, KeyPathMutable};
pub use navigable::{IndexNavigable, Navigable};

pub trait AsPatch {
    fn as_patch(&self) -> Patch;
}

/// Represents a command to the bindings to update their state
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Patch {
    #[serde(rename_all = "camelCase")]
    Splice {
        /// the keypath to the list to update
        key_path: serde_json::Value,
        /// the values to insert
        value: Vec<serde_json::Value>,
        /// position to insert the new value
        start: usize,
        /// number of existing items to replace
        replace: usize,
    },
    #[serde(rename_all = "camelCase")]
    Update {
        /// the keypath to the value to update
        key_path: serde_json::Value,
        /// the new value
        value: serde_json::Value,
    },
}

/// Represents a change to the state in the core
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Change<Root, T: Serialize> {
    Splice {
        /// the keypath to the list to update
        key_path: KeyPath<Root, Vec<T>>,
        /// the values to insert
        value: Vec<T>,
        /// position to insert the new value
        start: usize,
        /// number of existing items to replace
        replace: usize,
    },
    Update {
        /// the keypath to the value to update
        key_path: KeyPath<Root, T>,
        /// the new value
        value: T,
    },
}

impl<Root, T> Change<Root, T>
where
    Root: 'static,
    T: Serialize + 'static,
{
    pub fn update(key_path: KeyPath<Root, T>, value: T) -> ChangeOf<Root> {
        Change::Update { key_path, value }.into()
    }

    pub fn splice(
        key_path: KeyPath<Root, Vec<T>>,
        value: Vec<T>,
        start: usize,
        replace: usize,
    ) -> ChangeOf<Root> {
        Change::Splice {
            key_path,
            value,
            start,
            replace,
        }
        .into()
    }
}

impl<Root, T: Serialize + 'static> AsPatch for Change<Root, T> {
    fn as_patch(&self) -> Patch {
        match self {
            Change::Update { key_path, value } => Patch::Update {
                key_path: serde_json::to_value(key_path.path.clone())
                    .expect("Failed to serialize keypath"),
                value: serde_json::to_value(value).expect("Failed to serialize value"),
            },
            Change::Splice {
                key_path,
                value,
                start,
                replace,
            } => Patch::Splice {
                key_path: serde_json::to_value(key_path.path.clone())
                    .expect("Failed to serialize keypath"),
                value: value
                    .iter()
                    .map(|v| serde_json::to_value(v).expect("Failed to serialize value"))
                    .collect(),
                start: *start,
                replace: *replace,
            },
        }
    }
}

impl PartialEq for dyn AsPatch {
    fn eq(&self, other: &Self) -> bool {
        self.as_patch() == other.as_patch()
    }
}

impl std::fmt::Debug for dyn AsPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_patch())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChangeOf<Root> {
    Splice {
        key_path: KeyPathFrom<Root>,
        value: Vec<serde_json::Value>,
        start: usize,
        replace: usize,
    },
    Update {
        key_path: KeyPathFrom<Root>,
        value: serde_json::Value,
    },
}

impl<Root: 'static> ChangeOf<Root> {
    pub fn rebase<Base>(&self, base: &KeyPath<Base, Root>) -> ChangeOf<Base> {
        match self {
            ChangeOf::Update { key_path, value } => ChangeOf::Update {
                key_path: key_path.prepending(base),
                value: value.clone(),
            },
            ChangeOf::Splice {
                key_path,
                value,
                start,
                replace,
            } => ChangeOf::Splice {
                key_path: key_path.prepending(base),
                value: value.clone(),
                start: *start,
                replace: *replace,
            },
        }
    }

    pub fn downcast<T: Serialize + DeserializeOwned>(&self) -> Option<Change<Root, T>> {
        match self {
            ChangeOf::Update { key_path, value } => {
                let value = serde_json::from_value(value.clone()).ok()?;
                let key_path = key_path.downcast();

                Some(Change::Update { key_path, value })
            }
            ChangeOf::Splice {
                key_path,
                value,
                start,
                replace,
            } => {
                let value = value
                    .iter()
                    .map(|v| serde_json::from_value(v.clone()).ok())
                    .collect::<Option<Vec<T>>>()?;
                let key_path = key_path.downcast();

                Some(Change::Splice {
                    key_path,
                    value,
                    start: *start,
                    replace: *replace,
                })
            }
        }
    }

    pub fn key_path(&self) -> &KeyPathFrom<Root> {
        match self {
            ChangeOf::Update { key_path, .. } => key_path,
            ChangeOf::Splice { key_path, .. } => key_path,
        }
    }
}

impl<Root> AsPatch for ChangeOf<Root> {
    fn as_patch(&self) -> Patch {
        match self {
            ChangeOf::Update { key_path, value } => Patch::Update {
                key_path: serde_json::to_value(key_path.path.clone())
                    .expect("Failed to serialize keypath"),
                value: value.clone(),
            },
            ChangeOf::Splice {
                key_path,
                value,
                start,
                replace,
            } => Patch::Splice {
                key_path: serde_json::to_value(key_path.path.clone())
                    .expect("Failed to serialize keypath"),
                value: value.clone(),
                start: *start,
                replace: *replace,
            },
        }
    }
}

impl<Root, T: Serialize> From<Change<Root, T>> for ChangeOf<Root> {
    fn from(value: Change<Root, T>) -> Self {
        match value {
            Change::Update { key_path, value } => ChangeOf::Update {
                key_path: key_path.into(),
                value: serde_json::to_value(value).expect("Failed to serialize value"),
            },
            Change::Splice {
                key_path,
                value,
                start,
                replace,
            } => ChangeOf::Splice {
                key_path: key_path.into(),
                value: value
                    .iter()
                    .map(|v| serde_json::to_value(v).expect("Failed to serialize value"))
                    .collect(),
                start,
                replace,
            },
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod poc {
    use super::*;
    use crate::macros::Navigable;

    #[derive(Navigable, PartialEq, Debug)]
    struct Thing {
        a: usize,
        b: String,
    }

    #[derive(Navigable, PartialEq, Debug)]
    struct Other {
        thing: Thing,
        different_thing: Thing,
    }

    enum Edit {
        A(usize),
        _B(String),
    }

    fn change_thing(edit: Edit) -> ChangeOf<Thing> {
        match edit {
            Edit::A(a) => Change::update(keypath![Thing: a], a),
            Edit::_B(b) => Change::update(keypath![Thing: b], b),
        }
    }

    #[test]
    fn rebasing_changes() {
        let change = change_thing(Edit::A(2));

        let rebased = change.rebase(&keypath![Other: thing]);
        let different_rebased = change.rebase(&keypath![Other: different_thing]);

        let ChangeOf::Update { key_path, .. } = rebased else {
            panic!("Expected an update");
        };

        assert_eq!(key_path, keypath![Other: thing.a]);

        let ChangeOf::Update { key_path, .. } = different_rebased else {
            panic!("Expected an update");
        };

        assert_eq!(key_path, keypath![Other: different_thing.a]);
    }

    #[test]
    fn downcasting_changes() {
        let change = change_thing(Edit::A(2));

        let expected_change = Change::Update {
            key_path: keypath![Other: thing.a],
            value: 2,
        };
        let actual_change = change
            .rebase(&keypath![Other: thing])
            .downcast()
            .expect("Failed to downcast change");

        assert_eq!(expected_change, actual_change);
    }
}
