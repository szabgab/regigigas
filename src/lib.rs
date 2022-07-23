mod err;
mod nsid;
pub use err::*;
pub use nsid::*;

use ahash::{AHashMap, AHashSet};
use id_arena::{Arena, ArenaBehavior, DefaultArenaBehavior};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

type ArenaID<T> = <DefaultArenaBehavior<T> as ArenaBehavior>::Id;

pub struct Registry<T> {
    arena: Arena<(T, NamespacedID), DefaultArenaBehavior<T>>,
    nsid_map: AHashMap<NamespacedID, ArenaID<T>>,

    /// We LIE and tell it this can accept a thing called a "CatWrapper"
    /// this is to prevent needing horrible ArenaId<AHashSet< ... >>
    category_arena:
        Arena<(AHashSet<ArenaID<T>>, NamespacedID), DefaultArenaBehavior<CatWrapper<T>>>,
    category_nsid_map: AHashMap<NamespacedID, ArenaID<CatWrapper<T>>>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            nsid_map: AHashMap::new(),

            category_arena: Arena::new(),
            category_nsid_map: AHashMap::new(),
        }
    }

    /// Register something new with this registry.
    pub fn register(
        &mut self,
        entry: T,
        nsid: NamespacedID,
    ) -> Result<RegistryHandle<T>, ErrAlreadyRegistered> {
        if self.nsid_map.contains_key(&nsid) {
            return Err(ErrAlreadyRegistered);
        }
        let id = self.arena.alloc((entry, nsid));
        let handle = RegistryHandle::new(id, nsid);
        self.nsid_map.insert(nsid, handle.id);

        Ok(handle)
    }

    /// Create a new category with the specified elements.
    ///
    /// Duplicates in the iterator are ignored.
    pub fn register_category(
        &mut self,
        nsid: NamespacedID,
        entries: impl IntoIterator<Item = RegistryHandle<T>>,
    ) -> Result<CategoryHandle<T>, ErrCategoryAlreadyRegistered> {
        if self.category_nsid_map.contains_key(&nsid) {
            return Err(ErrCategoryAlreadyRegistered);
        }
        let set = entries.into_iter().map(|handle| handle.id).collect();
        let id = self.category_arena.alloc((set, nsid));
        let handle = CategoryHandle::new(id, nsid);
        self.category_nsid_map.insert(nsid, handle.id);

        Ok(handle)
    }

    /// Create a new empty category.
    pub fn register_empty_category(
        &mut self,
        nsid: NamespacedID,
    ) -> Result<CategoryHandle<T>, ErrCategoryAlreadyRegistered> {
        self.register_category(nsid, std::iter::empty())
    }

    /// Insert another element into this category.
    ///
    /// Duplicates are ignored.
    pub fn insert_into_category(&mut self, category: CategoryHandle<T>, entry: RegistryHandle<T>) {
        let set = &mut self.category_arena.get_mut(category.id).unwrap().0;
        set.insert(entry.id);
    }

    /// Insert many elements into this category. Duplicates are ignored.
    pub fn insert_many_into_category(
        &mut self,
        category: CategoryHandle<T>,
        entries: impl IntoIterator<Item = RegistryHandle<T>>,
    ) {
        let set = &mut self.category_arena.get_mut(category.id).unwrap().0;
        set.extend(entries.into_iter().map(|e| e.id));
    }

    /// Remove the given element from this category. Return whether it was actually removed or not.
    pub fn remove_from_category(
        &mut self,
        category: CategoryHandle<T>,
        entry: RegistryHandle<T>,
    ) -> bool {
        let set = &mut self.category_arena.get_mut(category.id).unwrap().0;
        set.remove(&entry.id)
    }

    /// Look up something from its handle.
    ///
    /// Because we have its handle, we know that we will always be able to get whatever it is out of the registry,
    /// so we return `&T` directly instead of an `Option<&T>`.
    pub fn lookup(&self, handle: RegistryHandle<T>) -> &T {
        &self.arena.get(handle.id).unwrap().0
    }

    /// Look up whatever NSID is associated with the handle..
    ///
    /// Because we have its handle, we know that we will always be able to get whatever it is out of the registry,
    /// so we return `&T` directly instead of an `Option<&T>`.
    pub fn get_nsid(&self, handle: RegistryHandle<T>) -> NamespacedID {
        self.arena.get(handle.id).unwrap().1
    }

    /// Look up something by a NSID, which may or may not actually be in here.
    pub fn lookup_by_nsid(&self, nsid: NamespacedID) -> Option<&T> {
        let id = self.nsid_map.get(&nsid)?;
        Some(&self.arena.get(*id).unwrap().0)
    }

    /// If this is a known NSID, turn it into a real `RegistryHandle`.
    pub fn validate_nsid(&self, nsid: NamespacedID) -> Option<RegistryHandle<T>> {
        let id = self.nsid_map.get(&nsid)?;
        Some(RegistryHandle::new(*id, nsid))
    }

    /// Iterate over everything in this registry.
    pub fn iter(&self) -> impl Iterator<Item = (&T, RegistryHandle<T>)> {
        self.arena
            .iter()
            .map(|(id, (x, nsid))| (x, RegistryHandle::new(id, *nsid)))
    }

    /// Look up all the elements in the given category.
    pub fn lookup_category(
        &self,
        category: CategoryHandle<T>,
    ) -> impl Iterator<Item = (&T, RegistryHandle<T>)> {
        let set = &self.category_arena.get(category.id).unwrap().0;
        set.iter().map(|id| {
            let (out, nsid) = self.arena.get(*id).unwrap();
            (out, RegistryHandle::new(*id, *nsid))
        })
    }

    /// Look up all the elements in the given category by its NSID.
    ///
    /// Returns `None` if that wasn't a recognized NSID.
    pub fn lookup_category_by_nsid(
        &self,
        nsid: NamespacedID,
    ) -> Option<impl Iterator<Item = (&T, RegistryHandle<T>)>> {
        let id = self.category_nsid_map.get(&nsid)?;
        let set = &self.category_arena.get(*id).unwrap().0;
        Some(set.iter().map(move |id| {
            let (out, _) = self.arena.get(*id).unwrap();
            (out, RegistryHandle::new(*id, nsid))
        }))
    }

    /// If this is a known NSID for a category, turn it into a real `CategoryHandle`.
    pub fn validate_category_nsid(&self, nsid: NamespacedID) -> Option<CategoryHandle<T>> {
        let id = self.category_nsid_map.get(&nsid)?;
        Some(CategoryHandle::new(*id, nsid))
    }

    /// Return if this entry is of the given category.
    pub fn is_in_category(&self, entry: RegistryHandle<T>, category: CategoryHandle<T>) -> bool {
        let set = &self.category_arena.get(category.id).unwrap().0;
        set.contains(&entry.id)
    }
}

impl<T: Default> Registry<T> {
    /// Register something new from this registry that we can auto-generate.
    /// This is handy for things that have no interesting info other than their identity.
    pub fn register_default(
        &mut self,
        nsid: NamespacedID,
    ) -> Result<RegistryHandle<T>, ErrAlreadyRegistered> {
        self.register(Default::default(), nsid)
    }
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Lightweight handle to an entry in a registry.
pub struct RegistryHandle<T> {
    id: ArenaID<T>,
    nsid: NamespacedID,
}

impl<T> RegistryHandle<T> {
    fn new(handle: ArenaID<T>, nsid: NamespacedID) -> Self {
        Self { id: handle, nsid }
    }

    pub fn get_nsid(&self) -> NamespacedID {
        self.nsid
    }
}

// Manual impls cause it doesn't believe me it doesn't actually own a T
impl<T> Clone for RegistryHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            nsid: self.nsid,
        }
    }
}

impl<T> Copy for RegistryHandle<T> {}

impl<T> Hash for RegistryHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> PartialEq for RegistryHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for RegistryHandle<T> {}

impl<T> Debug for RegistryHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RegistryHandle").field(&self.nsid).finish()
    }
}

/// Lightweight handle to a *category* of entries in a registry.
pub struct CategoryHandle<T> {
    id: ArenaID<CatWrapper<T>>,
    nsid: NamespacedID,
}

impl<T> CategoryHandle<T> {
    fn new(handle: ArenaID<CatWrapper<T>>, nsid: NamespacedID) -> Self {
        Self { id: handle, nsid }
    }

    pub fn get_nsid(&self) -> NamespacedID {
        self.nsid
    }
}
impl<T> Clone for CategoryHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            nsid: self.nsid,
        }
    }
}

impl<T> Copy for CategoryHandle<T> {}

impl<T> Hash for CategoryHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> PartialEq for CategoryHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for CategoryHandle<T> {}

impl<T> Debug for CategoryHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CategoryHandle").field(&self.nsid).finish()
    }
}

/// Internal struct to help differentiate handles to the arena itself and to the category arena.
struct CatWrapper<T>(PhantomData<T>);
