use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use uuid::Uuid;

/// Universal Identifier (uuidv4).
#[derive(Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct ID<T> {
    id: Uuid,
    #[serde(skip_serializing, skip_deserializing)]
    _phantom: PhantomData<T>,
}

impl<T> ID<T> {
    /// Creates new identifier.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets underlying UUID object.
    #[inline]
    pub fn uuid(&self) -> Uuid {
        self.id
    }
}

impl<T> Default for ID<T> {
    #[inline]
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            _phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for ID<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<T> ToString for ID<T> {
    #[inline]
    fn to_string(&self) -> String {
        format!("ID({})", self.id)
    }
}

impl<T> Hash for ID<T> {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state)
    }
}

impl<T> PartialEq for ID<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for ID<T> {}
impl<T> Copy for ID<T> where T: Clone {}

impl<T> PartialOrd for ID<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl<T> Ord for ID<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}
