use bevy_ecs::prelude::*;
use bevy_ecs::query::{QueryData, QueryFilter, QueryItem};
use moonshine_kind::{prelude::*, Any};

use crate::{Object, ObjectName, ObjectRebind, ObjectRef, Objects};

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
            .unwrap_or_else(|| self.rebind_any(self.entity()))
    }

    /// Returns true if this object is the root of its hierarchy.
    fn is_root(&self) -> bool {
        self.parent().is_none()
    }

    /// Returns true if this object is a child of another.
    fn is_child(&self) -> bool {
        self.parent().is_some()
    }

    /// Returns true if this object is a child of the given parent entity.
    fn is_child_of(&self, parent: Entity) -> bool {
        self.parent()
            .is_some_and(|object| object.entity() == parent)
    }

    /// Returns an iterator over all the children of this object.
    fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    /// Returns true if this object has any children.
    fn has_children(&self) -> bool {
        self.children().next().is_some()
    }

    /// Queries the children of this object.
    fn query_children<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
        self.children()
            .filter_map(move |object| query.get(object.entity()).ok())
    }

    /// Returns an iterator over all children of this object which are of the given kind.
    fn children_of_kind<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.children()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Returns the first child of this object which is of the given kind, if it exists.
    fn find_child_of_kind<U: Kind>(&self, objects: &Objects<U>) -> Option<Self::Rebind<U>> {
        self.children()
            .find_map(|object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Returns an iterator over all ancestors of this object.
    fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    /// Returns an iterator over this object and all its ancestors.
    fn self_and_ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        std::iter::once(self.rebind_any(self.entity())).chain(self.ancestors())
    }

    /// Returns true if this object is an ancestor of the given entity.
    fn is_ancestor_of(&self, entity: Entity) -> bool
    where
        Self::Rebind<Any>: ObjectHierarchy<Any>,
    {
        self.rebind_any(entity)
            .ancestors()
            .any(|ancestor| ancestor.entity() == self.entity())
    }

    /// Queries the ancestors of this object.
    fn query_ancestors<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
        self.ancestors().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Queries this object and its ancestors.
    fn query_self_and_ancestors<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'a, Q::ReadOnly>> + 'a {
        self.self_and_ancestors().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Returns an iterator over all ancestors of this object which are of the given kind.
    fn ancestors_of_kind<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.ancestors()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Returns an iterator over this object and all its ancestors which are of the given kind.
    fn self_and_ancestors_of_kind<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.self_and_ancestors()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Returns the first ancestor of this object which is of the given kind, if it exists.
    fn find_ancestor_of_kind<U: Kind>(&self, objects: &Objects<U>) -> Option<Self::Rebind<U>> {
        self.ancestors()
            .find_map(|object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Returns an iterator over all descendants of this object in breadth-first order.
    fn descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    /// Returns an iterator over all descendants of this object in depth-first order.
    fn descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    /// Returns an iterator over this object and all its descendants in breadth-first order.
    fn self_and_descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        std::iter::once(self.rebind_any(self.entity())).chain(self.descendants_wide())
    }

    /// Returns an iterator over this object and all its descendants in depth-first order.
    fn self_and_descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        std::iter::once(self.rebind_any(self.entity())).chain(self.descendants_deep())
    }

    /// Returns true if this object is a descendant of the given entity.
    fn is_descendant_of(&self, entity: Entity) -> bool
    where
        Self::Rebind<Any>: ObjectHierarchy<Any>,
    {
        self.ancestors().any(|ancestor| ancestor.entity() == entity)
    }

    /// Queries the descendants of this object in breadth-first order.
    fn descendants_of_kind_wide<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.descendants_wide()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Queries the descendants of this object in depth-first order.
    fn descendants_of_kind_deep<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.descendants_deep()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Queries this object and all its descendants in breadth-first order.
    fn self_and_descendants_of_kind_wide<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.self_and_descendants_wide()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Queries this object and all its descendants in depth-first order.
    fn self_and_descendants_of_kind_deep<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.self_and_descendants_deep()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Queries the descendants of this object in breadth-first order.
    fn query_descendants_wide<'a, 'q, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'q Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'q, Q::ReadOnly>> + 'a
    where
        'q: 'a,
    {
        self.descendants_wide().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Queries the descendants of this object in depth-first order.
    fn query_descendants_deep<'a, 'q, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'q Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'q, Q::ReadOnly>> + 'a
    where
        'q: 'a,
    {
        self.descendants_deep().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Queries this object and all its descendants in breadth-first order.
    fn query_self_and_descendants_wide<'a, 'q, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'q Query<Q, F>,
    ) -> impl Iterator<Item = QueryItem<'q, Q::ReadOnly>> + 'a
    where
        'q: 'a,
    {
        self.self_and_descendants_wide().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Queries this object and all its descendants in depth-first order.
    fn query_self_and_descendants_deep<'a, 'q, Q: QueryData>(
        &'a self,
        query: &'q Query<Q>,
    ) -> impl Iterator<Item = QueryItem<'q, Q::ReadOnly>> + 'a
    where
        'q: 'a,
    {
        self.self_and_descendants_deep().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    /// Returns the first descendant of this object (breadth-first order) which is of the given kind, if it exists.
    fn find_descendant_of_kind_wide<U: Kind>(
        &self,
        objects: &Objects<U>,
    ) -> Option<Self::Rebind<U>> {
        self.descendants_wide()
            .find_map(|object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    /// Returns the first descendant of this object (depth-first order) which is of the given kind, if it exists.
    fn find_descendant_of_kind_deep<U: Kind>(
        &self,
        objects: &Objects<U>,
    ) -> Option<Self::Rebind<U>> {
        self.descendants_deep()
            .find_map(|object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
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
            .map(|entity| self.rebind_any(entity))
    }

    fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.hierarchy
            .children(self.entity())
            .map(|entity| self.rebind_any(entity))
    }

    fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.hierarchy
            .ancestors(self.entity())
            .map(|entity| self.rebind_any(entity))
    }

    fn descendants_wide(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.hierarchy
            .descendants_wide(self.entity())
            .map(|entity| self.rebind_any(entity))
    }

    fn descendants_deep(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.hierarchy
            .descendants_deep(self.entity())
            .map(|entity| self.rebind_any(entity))
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
        return None;
    } else if let Some(child) = curr
        .children()
        .find(|part| part.name().is_some_and(|name| name == head))
    {
        find_by_path(child, tail)
    } else {
        None
    }
}
