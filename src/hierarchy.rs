use bevy_ecs::prelude::*;
use bevy_ecs::query::{QueryData, QueryFilter, QueryItem};
use moonshine_kind::{prelude::*, Any};
use moonshine_util::hierarchy::{WorldDescendantsDeepIter, WorldDescendantsWideIter};

use crate::{Object, ObjectName, ObjectRebind, ObjectRef, ObjectWorldRef, Objects};

/// [`Object`] methods related to hierarchy traversal.
///
/// These methods are available to any [`Object<T>`] or [`ObjectRef<T>`] type.
pub trait ObjectHierarchy<T: Kind = Any>: ObjectRebind<T> + ObjectName {
    /// Returns the parent of this object, if it exists.
    fn parent(&self) -> Option<Self::Rebind<Any>>;

    /// Returns the root of this object's hierarchy.
    fn root(&self) -> Self::Rebind<Any> {
        self.ancestors()
            .last()
            // SAFE: If this object is valid, then so must be its root
            .unwrap_or_else(|| unsafe { self.rebind_any(self.entity()) })
    }

    /// Returns true if this object is the root of its hierarchy.
    fn is_root(&self) -> bool {
        self.parent().is_none()
    }

    /// Returns true if this object has the same root as another.
    fn is_related_to<U: Kind>(&self, other: &impl ObjectHierarchy<U>) -> bool {
        self.root().entity() == other.root().entity()
    }

    /// Returns true if this object is a child of another.
    fn is_child(&self) -> bool {
        self.parent().is_some()
    }

    /// Returns true if this object is a child of the given [`Entity`].
    fn is_child_of(&self, entity: Entity) -> bool {
        self.parent()
            .is_some_and(|object| object.entity() == entity)
    }

    /// Iterates over all children of this object.
    fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    /// Returns true if this object has any children.
    fn has_children(&self) -> bool {
        self.children().next().is_some()
    }

    /// Iterates over all children of this object which match the given [`Query`].
    fn query_children<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'a, 'a, Q::ReadOnly>> + 'a {
        self.children()
            .filter_map(move |object| query.get(object.entity()).ok())
    }

    /// Iterates over all children of this object which match the given [`Kind`].
    fn children_of_kind<'w, 's, 'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'w, 's, U>,
    ) -> impl Iterator<Item = Object<'w, 's, 'a, U>> + 'a {
        self.children()
            .filter_map(move |object| objects.get(object.entity()).ok())
    }

    /// Returns the first child of this object which matches the given kind, if it exists.
    fn find_child_of_kind<'w, 's, 'a, U: Kind>(
        &self,
        objects: &'a Objects<'w, 's, U>,
    ) -> Option<Object<'w, 's, 'a, U>> {
        self.children()
            .find_map(|object| objects.get(object.entity()).ok())
    }

    /// Iterates over all ancestors of this object.
    fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    /// Iterates over this object, followed by all of its ancestors.
    fn self_and_ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        std::iter::Iterator::chain(std::iter::once(self.as_any()), self.ancestors())
    }

    /// Returns true if this object is an ancestor of another.
    fn is_ancestor_of<U: Kind>(&self, other: &impl ObjectHierarchy<U>) -> bool
    where
        Self::Rebind<Any>: ObjectHierarchy<Any>,
    {
        other
            .ancestors()
            .any(|ancestor| ancestor.entity() == self.entity())
    }

    /// Iterates over all ancestors of this object which match the given [`Query`].
    fn query_ancestors<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'a, 'a, Q::ReadOnly>> + 'a {
        self.ancestors().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Iterates over this object, followed by all its ancestors which match the given [`Query`].
    fn query_self_and_ancestors<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'a, 'a, Q::ReadOnly>> + 'a {
        self.self_and_ancestors().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Iterates over all ancestors of this object which match the given [`Kind`].
    fn ancestors_of_kind<'w, 's, 'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'w, 's, U>,
    ) -> impl Iterator<Item = Object<'w, 's, 'a, U>> {
        self.ancestors()
            .filter_map(move |object| objects.get(object.entity()).ok())
    }

    /// Iterates over this object, followed by all its ancestors which match the given [`Kind`].
    fn self_and_ancestors_of_kind<'w, 's, 'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'w, 's, U>,
    ) -> impl Iterator<Item = Object<'w, 's, 'a, U>> {
        self.self_and_ancestors()
            .filter_map(move |object| objects.get(object.entity()).ok())
    }

    /// Returns the first ancestor of this object which matches the given [`Kind`], if it exists.
    fn find_ancestor_of_kind<'w, 's, 'a, U: Kind>(
        &self,
        objects: &'a Objects<'w, 's, U>,
    ) -> Option<Object<'w, 's, 'a, U>> {
        self.ancestors()
            .find_map(|object| objects.get(object.entity()).ok())
    }

    /// Iterates over all descendants of this object in breadth-first order.
    fn descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    /// Iterates over all descendants of this object in depth-first order.
    fn descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    /// Iterates over this object and all its descendants in breadth-first order.
    fn self_and_descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        std::iter::Iterator::chain(std::iter::once(self.as_any()), self.descendants_wide())
    }

    /// Iterates over this object and all its descendants in depth-first order.
    fn self_and_descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        std::iter::Iterator::chain(std::iter::once(self.as_any()), self.descendants_deep())
    }

    /// Returns true if this object is a descendant of the given entity.
    fn is_descendant_of(&self, entity: Entity) -> bool
    where
        Self::Rebind<Any>: ObjectHierarchy<Any>,
    {
        self.ancestors().any(|ancestor| ancestor.entity() == entity)
    }

    /// Iterates over all descendants of this object which match the given [`Kind`] in breadth-first order.
    fn descendants_of_kind_wide<'w, 's, 'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'w, 's, U>,
    ) -> impl Iterator<Item = Object<'w, 's, 'a, U>> {
        self.descendants_wide()
            .filter_map(move |object| objects.get(object.entity()).ok())
    }

    /// Iterates over all descendants of this object which match the given [`Kind`] in depth-first order.
    fn descendants_of_kind_deep<'w, 's, 'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'w, 's, U>,
    ) -> impl Iterator<Item = Object<'w, 's, 'a, U>> {
        self.descendants_deep()
            .filter_map(move |object| objects.get(object.entity()).ok())
    }

    /// Iterates over this object, followed by all its descendants which match the given [`Kind`] in breadth-first order.
    fn self_and_descendants_of_kind_wide<'w, 's, 'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'w, 's, U>,
    ) -> impl Iterator<Item = Object<'w, 's, 'a, U>> {
        self.self_and_descendants_wide()
            .filter_map(move |object| objects.get(object.entity()).ok())
    }

    /// Iterates over this object, followed by all its descendants which match the given [`Kind`] in depth-first order.
    fn self_and_descendants_of_kind_deep<'w, 's, 'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'w, 's, U>,
    ) -> impl Iterator<Item = Object<'w, 's, 'a, U>> {
        self.self_and_descendants_deep()
            .filter_map(move |object| objects.get(object.entity()).ok())
    }

    /// Iterates over all descendants of this object which match the given [`Query`] in breadth-first order.
    fn query_descendants_wide<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'a, 'a, Q::ReadOnly>> + 'a {
        self.descendants_wide().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Iterates over all descendants of this object which match the given [`Kind`] in depth-first order.
    fn query_descendants_deep<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'a, 'a, Q::ReadOnly>> + 'a {
        self.descendants_deep().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Iterates over this object, followed by all its descendants which match the given [`Query`] in breadth-first order.
    fn query_self_and_descendants_wide<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'a, 'a, Q::ReadOnly>> + 'a {
        self.self_and_descendants_wide().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Iterates over this object, followed by all its descendants which match the given [`Query`] in depth-first order.
    fn query_self_and_descendants_deep<'a, Q: QueryData>(
        &'a self,
        query: &'a Query<Q>,
    ) -> impl Iterator<Item = QueryItem<'a, 'a, Q::ReadOnly>> + 'a {
        self.self_and_descendants_deep().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Returns the first descendant of this object (breadth-first order) which matches the given [`Kind`], if it exists.
    fn find_descendant_of_kind_wide<'w, 's, 'a, U: Kind>(
        &self,
        objects: &'a Objects<'w, 's, U>,
    ) -> Option<Object<'w, 's, 'a, U>> {
        self.descendants_wide()
            .find_map(|object| objects.get(object.entity()).ok())
    }

    /// Returns the first descendant of this object (depth-first order) which matches the given [`Kind`], if it exists.
    fn find_descendant_of_kind_deep<'w, 's, 'a, U: Kind>(
        &self,
        objects: &'a Objects<'w, 's, U>,
    ) -> Option<Object<'w, 's, 'a, U>> {
        self.descendants_deep()
            .find_map(|object| objects.get(object.entity()).ok())
    }

    /// Returns the path to this object.
    fn path(&self) -> String {
        // TODO: Can this be optimized?
        let mut tokens = self
            .self_and_ancestors()
            .map(|ancestor| ancestor.name().unwrap_or_default().to_string())
            .collect::<Vec<_>>();
        tokens.reverse();
        tokens.join("/")
    }

    /// Attempts to find an object by its path, relative to this one.
    ///
    /// # Usage
    ///
    /// An **Object Path** is a string of object names separated by slashes which represents
    /// the path to an object within a hierarchy.
    ///
    /// In additional to object names, the path may contain the following special characters:
    ///   - `.` represents this object.
    ///   - `..` represents the parent object.
    ///   - `*` represents any child object.
    ///
    /// Note that this method of object search is relatively slow, and should be reserved for
    /// when performance is not the top priority, such as during initialization or prototyping.
    ///
    /// Instead, prefer to use [`Component`] to tag your entities and [`Query`] them instead, if possible.
    ///
    /// # Safety
    /// This method is somewhat experimental with plans for future expansion.
    /// Please [report](https://github.com/Zeenobit/moonshine_object/issues) any bugs you encounter or features you'd like.
    fn find_by_path(&self, path: impl AsRef<str>) -> Option<Self::Rebind<Any>>;
}

impl<T: Kind> ObjectHierarchy<T> for Object<'_, '_, '_, T> {
    fn parent(&self) -> Option<Self::Rebind<Any>> {
        self.hierarchy
            .parent(self.entity())
            // SAFE: If this object is valid, then so must be its parent
            .map(|entity| unsafe { self.rebind_any(entity) })
    }

    fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.hierarchy
            .children(self.entity())
            // SAFE: We assume Bevy removes invalid children
            .map(|entity| unsafe { self.rebind_any(entity) })
    }

    fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.hierarchy
            .ancestors(self.entity())
            // SAFE: If this object is valid, then so must be its ancestors
            .map(|entity| unsafe { self.rebind_any(entity) })
    }

    fn descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.hierarchy
            .descendants_wide(self.entity())
            // SAFE: We assume Bevy removes invalid children
            .map(|entity| unsafe { self.rebind_any(entity) })
    }

    fn descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.hierarchy
            .descendants_deep(self.entity())
            // SAFE: We assume Bevy removes invalid children
            .map(|entity| unsafe { self.rebind_any(entity) })
    }

    fn find_by_path(&self, path: impl AsRef<str>) -> Option<Self::Rebind<Any>> {
        let tail: Vec<&str> = path.as_ref().split('/').collect();
        find_by_path(self.cast_into_any(), &tail)
    }
}

impl<T: Kind> ObjectHierarchy<T> for ObjectRef<'_, '_, '_, T> {
    fn parent(&self) -> Option<Self::Rebind<Any>> {
        self.1.parent().map(|object| ObjectRef(self.0, object))
    }

    fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.1.children().map(|object| ObjectRef(self.0, object))
    }

    fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.1.ancestors().map(|object| ObjectRef(self.0, object))
    }

    fn descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.1
            .descendants_wide()
            .map(|object| ObjectRef(self.0, object))
    }

    fn descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.1
            .descendants_deep()
            .map(|object| ObjectRef(self.0, object))
    }

    fn find_by_path(&self, path: impl AsRef<str>) -> Option<Self::Rebind<Any>> {
        self.1
            .find_by_path(path)
            .map(|object| ObjectRef(self.0, object))
    }
}

impl<T: Kind> ObjectHierarchy<T> for ObjectWorldRef<'_, T> {
    fn parent(&self) -> Option<Self::Rebind<Any>> {
        let &ChildOf(parent) = self.world.get(self.entity())?;
        // SAFE: If this object is valid, then so must be its parent
        Some(unsafe { self.rebind_any(parent) })
    }

    fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.world
            .get::<Children>(self.entity())
            .into_iter()
            .flat_map(|children| children.iter())
            // SAFE: Assume Bevy removes invalid children
            .map(|entity| unsafe { self.rebind_any(entity) })
    }

    fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        std::iter::successors(self.parent(), |current| current.parent())
    }

    fn descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        WorldDescendantsWideIter::new(self.world, self.entity())
            // SAFE: Assume Bevy resomves invalid descendants
            .map(|entity| unsafe { self.rebind_any(entity) })
    }

    fn descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        WorldDescendantsDeepIter::new(self.world, self.entity())
            // SAFE: Assume Bevy resomves invalid descendants
            .map(|entity| unsafe { self.rebind_any(entity) })
    }

    fn find_by_path(&self, path: impl AsRef<str>) -> Option<Self::Rebind<Any>> {
        let tail: Vec<&str> = path.as_ref().split('/').collect();
        find_by_path(self.cast_into_any(), &tail)
    }
}

fn find_by_path<T: ObjectHierarchy<Rebind<Any> = T>>(
    curr: T,
    tail: &[&str],
) -> Option<T::Rebind<Any>> {
    if tail.is_empty() {
        return Some(curr);
    }

    let head = tail[0];
    let tail = &tail[1..];

    if head == "." || head.is_empty() {
        find_by_path(curr, tail)
    } else if head == ".." {
        if let Some(parent) = curr.parent() {
            find_by_path(parent, tail)
        } else {
            None
        }
    } else if head == "*" {
        for child in curr.children() {
            if let Some(result) = find_by_path(child, tail) {
                return Some(result);
            }
        }
        None
    } else if let Some(child) = curr
        .children()
        .find(|part| part.name().is_some_and(|name| name == head))
    {
        find_by_path(child, tail)
    } else {
        None
    }
}
