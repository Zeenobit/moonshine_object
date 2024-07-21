#![doc = include_str!("../README.md")]

use std::{fmt, ops::Deref};

use bevy_core::Name;
use bevy_ecs::{
    prelude::*,
    query::{QueryData, QueryEntityError, QueryFilter, QueryItem, QuerySingleError},
    system::SystemParam,
};
use moonshine_kind::prelude::*;
use moonshine_util::hierarchy::HierarchyQuery;

pub mod prelude {
    pub use super::{
        Object, ObjectCast, ObjectHierarchy, ObjectInstance, ObjectName, ObjectRebind, ObjectRef,
        Objects,
    };
}

pub use moonshine_kind::{Any, CastInto, Kind};

/// A [`SystemParam`] similar to [`Query`] which provides [`Object<T>`] access for its items.
#[derive(SystemParam)]
pub struct Objects<'w, 's, T = Any, F = ()>
where
    T: Kind,
    F: 'static + QueryFilter,
{
    pub instance: Query<'w, 's, Instance<T>, F>,
    pub hierarchy: HierarchyQuery<'w, 's>,
    pub name: Query<'w, 's, &'static Name>,
}

impl<'w, 's, T, F> Objects<'w, 's, T, F>
where
    T: Kind,
    F: 'static + QueryFilter,
{
    /// Iterates over all [`Object`]s of [`Kind`] `T` which match the [`QueryFilter`] `F`.
    pub fn iter(&self) -> impl Iterator<Item = Object<'w, 's, '_, T>> {
        self.instance.iter().map(|instance| Object {
            instance,
            hierarchy: &self.hierarchy,
            name: &self.name,
        })
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.instance.contains(entity)
    }

    pub fn iter_ref<'a>(
        &'a self,
        world: &'a World,
    ) -> impl Iterator<Item = ObjectRef<'w, 's, 'a, T>> {
        self.iter()
            .map(|object| ObjectRef(world.entity(object.entity()), object))
    }

    /// Gets the [`Object`] of [`Kind`] `T` from an [`Entity`], if it matches.
    pub fn get(&self, entity: Entity) -> Result<Object<'w, 's, '_, T>, QueryEntityError> {
        self.instance.get(entity).map(|instance| Object {
            instance,
            hierarchy: &self.hierarchy,
            name: &self.name,
        })
    }

    pub fn get_ref<'a>(&'a self, entity: EntityRef<'a>) -> Option<ObjectRef<'w, 's, 'a, T>> {
        Some(ObjectRef(entity, self.get(entity.id()).ok()?))
    }

    pub fn get_single(&self) -> Result<Object<'w, 's, '_, T>, QuerySingleError> {
        self.instance.get_single().map(|instance| Object {
            instance,
            hierarchy: &self.hierarchy,
            name: &self.name,
        })
    }

    pub fn get_single_ref<'a>(&'a self, entity: EntityRef<'a>) -> Option<ObjectRef<'w, 's, 'a, T>> {
        Some(ObjectRef(entity, self.get_single().ok()?))
    }

    /// Gets the [`Object`] of [`Kind`] `T` from an [`Instance`].
    ///
    /// # Safety
    /// Assumes `instance` is a valid [`Instance`] of [`Kind`] `T`.
    pub fn instance(&self, instance: Instance<T>) -> Object<'w, 's, '_, T> {
        self.get(instance.entity()).expect("instance must be valid")
    }
}

pub trait ObjectInstance<T: Kind> {
    fn instance(&self) -> Instance<T>;

    fn entity(&self) -> Entity {
        self.instance().entity()
    }
}

impl<'w, 's, 'a, T: Kind> ObjectInstance<T> for Object<'w, 's, 'a, T> {
    fn instance(&self) -> Instance<T> {
        self.instance
    }
}

impl<'w, 's, 'a, T: Kind> ObjectInstance<T> for ObjectRef<'w, 's, 'a, T> {
    fn instance(&self) -> Instance<T> {
        self.1.instance()
    }
}

pub trait ObjectName {
    fn name(&self) -> Option<&str>;
}

impl<'w, 's, 'a, T: Kind> ObjectName for Object<'w, 's, 'a, T> {
    fn name(&self) -> Option<&str> {
        self.name.get(self.entity()).ok().map(|name| name.as_str())
    }
}

impl<'w, 's, 'a, T: Kind> ObjectName for ObjectRef<'w, 's, 'a, T> {
    fn name(&self) -> Option<&str> {
        self.1.name()
    }
}

pub trait ObjectRebind<T: Kind>: ObjectInstance<T> {
    type Rebind<U: Kind>: ObjectInstance<U>;

    fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U>;

    fn rebind(&self, instance: Instance<T>) -> Self::Rebind<T> {
        self.rebind_as(instance)
    }

    fn rebind_any(&self, entity: Entity) -> Self::Rebind<Any> {
        self.rebind_as(Instance::from(entity))
    }
}

impl<'w, 's, 'a, T: Kind> ObjectRebind<T> for Object<'w, 's, 'a, T> {
    type Rebind<U: Kind> = Object<'w, 's, 'a, U>;

    fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U> {
        Object {
            instance,
            hierarchy: self.hierarchy,
            name: self.name,
        }
    }
}

impl<'w, 's, 'a, T: Kind> ObjectRebind<T> for ObjectRef<'w, 's, 'a, T> {
    type Rebind<U: Kind> = ObjectRef<'w, 's, 'a, U>;

    fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U> {
        ObjectRef(self.0, self.1.rebind_as(instance))
    }
}

pub trait ObjectCast<T: Kind>: ObjectInstance<T> + ObjectRebind<T> + Sized {
    fn cast_into<U: Kind>(self) -> Self::Rebind<U>
    where
        T: CastInto<U>,
    {
        self.rebind_as(self.instance().cast_into())
    }

    fn cast_into_any(self) -> Self::Rebind<Any> {
        self.rebind_as(self.instance().cast_into_any())
    }

    /// # Safety
    ///
    /// TODO
    unsafe fn cast_into_unchecked<U: Kind>(self) -> Self::Rebind<U> {
        self.rebind_as(self.instance().cast_into_unchecked())
    }
}

impl<T: Kind> ObjectCast<T> for Object<'_, '_, '_, T> {}

impl<T: Kind> ObjectCast<T> for ObjectRef<'_, '_, '_, T> {}

pub trait ObjectHierarchy<T: Kind>: ObjectRebind<T> {
    fn parent(&self) -> Option<Self::Rebind<Any>>;

    fn root(&self) -> Self::Rebind<Any> {
        self.ancestors()
            .last()
            .unwrap_or_else(|| self.rebind_any(self.entity()))
    }

    fn is_root(&self) -> bool {
        self.parent().is_none()
    }

    fn is_child(&self) -> bool {
        self.parent().is_some()
    }

    fn is_child_of(&self, parent: Entity) -> bool {
        self.parent()
            .is_some_and(|object| object.entity() == parent)
    }

    fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    fn has_children(&self) -> bool {
        self.children().next().is_some()
    }

    fn query_children<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<'_, '_, Q, F>,
    ) -> impl Iterator<Item = QueryItem<'_, Q::ReadOnly>> + 'a {
        self.children()
            .filter_map(move |object| query.get(object.entity()).ok())
    }

    fn children_of_kind<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'_, '_, U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.children()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    fn find_child_of_kind<U: Kind>(&self, objects: &Objects<'_, '_, U>) -> Option<Self::Rebind<U>> {
        self.children()
            .find_map(|object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    fn self_and_ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        std::iter::once(self.rebind_any(self.entity())).chain(self.ancestors())
    }

    fn is_ancestor_of(&self, entity: Entity) -> bool
    where
        Self::Rebind<Any>: ObjectHierarchy<Any>,
    {
        self.rebind_any(entity)
            .ancestors()
            .any(|ancestor| ancestor.entity() == self.entity())
    }

    fn query_ancestors<'a, Q: QueryData, F: QueryFilter>(
        &'a self,
        query: &'a Query<'_, '_, Q, F>,
    ) -> impl Iterator<Item = QueryItem<'_, Q::ReadOnly>> + 'a {
        self.ancestors().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    fn ancestors_of_kind<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'_, '_, U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.ancestors()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    fn find_ancestor_of_kind<U: Kind>(
        &self,
        objects: &Objects<'_, '_, U>,
    ) -> Option<Self::Rebind<U>> {
        self.ancestors()
            .find_map(|object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    fn descendants(&self) -> impl Iterator<Item = Self::Rebind<Any>>;

    fn self_and_descendants(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        std::iter::once(self.rebind_any(self.entity())).chain(self.descendants())
    }

    fn is_descendant_of(&self, entity: Entity) -> bool
    where
        Self::Rebind<Any>: ObjectHierarchy<Any>,
    {
        self.rebind_any(entity)
            .descendants()
            .any(|descendant| descendant.entity() == self.entity())
    }

    fn query_descendants<'a, Q: QueryData>(
        &'a self,
        query: &'a Query<'_, '_, Q>,
    ) -> impl Iterator<Item = QueryItem<'_, Q::ReadOnly>> + 'a {
        self.descendants().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    fn descendants_of_kind<'a, U: Kind>(
        &'a self,
        objects: &'a Objects<'_, '_, U>,
    ) -> impl Iterator<Item = Self::Rebind<U>> + 'a {
        self.descendants()
            .filter_map(move |object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }

    fn find_descendant_of_kind<U: Kind>(
        &self,
        objects: &Objects<'_, '_, U>,
    ) -> Option<Self::Rebind<U>> {
        self.descendants()
            .find_map(|object| objects.get(object.entity()).ok())
            .map(|object| self.rebind_as(object.instance()))
    }
}

impl<'w, 's, 'a, T: Kind> ObjectHierarchy<T> for Object<'w, 's, 'a, T> {
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

    fn descendants(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.hierarchy
            .descendants(self.entity())
            .map(|entity| self.rebind_any(entity))
    }
}

impl<'w, 's, 'a, T: Kind> ObjectHierarchy<T> for ObjectRef<'w, 's, 'a, T> {
    fn parent(&self) -> Option<Self::Rebind<Any>> {
        self.1.parent().map(|object| ObjectRef(self.0, object))
    }

    fn children(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.1.children().map(|object| ObjectRef(self.0, object))
    }

    fn ancestors(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.1.ancestors().map(|object| ObjectRef(self.0, object))
    }

    fn descendants(&self) -> impl Iterator<Item = Self::Rebind<Any>> {
        self.1.descendants().map(|object| ObjectRef(self.0, object))
    }
}

/// Represents an [`Entity`] of [`Kind`] `T` with hierarchy and name information.
pub struct Object<'w, 's, 'a, T: Kind = Any> {
    instance: Instance<T>,
    hierarchy: &'a HierarchyQuery<'w, 's>,
    name: &'a Query<'w, 's, &'static Name>,
}

impl<'w, 's, 'a, T: Kind> Object<'w, 's, 'a, T> {
    /// Creates a new [`Object<T>`] from an [`Object<Any>`].
    ///
    /// This is semantically equivalent to an unsafe downcast.
    ///
    /// # Safety
    /// Assumes `base` is of [`Kind`] `T`.
    pub unsafe fn from_base_unchecked(base: Object<'w, 's, 'a>) -> Self {
        Self {
            instance: base.instance.cast_into_unchecked(),
            hierarchy: base.hierarchy,
            name: base.name,
        }
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
    pub fn find_by_path(&self, path: impl AsRef<str>) -> Option<Object<'w, 's, 'a>> {
        let tail: Vec<&str> = path.as_ref().split('/').collect();
        find_by_path(self.cast_into_any(), &tail)
    }
}

impl<'w, 's, 'a, T: Component> Object<'w, 's, 'a, T> {
    pub fn from_base(world: &World, object: Object<'w, 's, 'a>) -> Option<Object<'w, 's, 'a, T>> {
        let entity = world.entity(object.entity());
        let instance = Instance::<T>::from_entity(entity)?;
        // SAFE: Entity was just checked to a valid instance of T.
        Some(object.rebind_as(instance))
    }
}

impl<T: Kind> Clone for Object<'_, '_, '_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Kind> Copy for Object<'_, '_, '_, T> {}

impl<T: Kind> From<Object<'_, '_, '_, T>> for Entity {
    fn from(object: Object<'_, '_, '_, T>) -> Self {
        object.entity()
    }
}

impl<T: Kind> From<Object<'_, '_, '_, T>> for Instance<T> {
    fn from(object: Object<'_, '_, '_, T>) -> Self {
        object.instance()
    }
}

impl<T: Kind> PartialEq for Object<'_, '_, '_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.instance == other.instance
    }
}

impl<T: Kind> Eq for Object<'_, '_, '_, T> {}

impl<T: Kind> fmt::Debug for Object<'_, '_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = f.debug_tuple(&T::debug_name());
        out.field(&self.entity());
        if let Some(name) = self.name() {
            out.field(&name);
        }
        out.finish()
    }
}

fn find_by_path<'w, 's, 'a>(curr: Object<'w, 's, 'a>, tail: &[&str]) -> Option<Object<'w, 's, 'a>> {
    if tail.is_empty() {
        return Some(curr);
    }

    let head = tail[0];
    let tail = &tail[1..];

    if head == "." || head.is_empty() {
        find_by_path(curr, tail)
    } else if head == ".." {
        if let Some(parent) = curr
            .hierarchy
            .parent(curr.entity())
            .map(|parent| curr.rebind_any(parent))
        {
            find_by_path(parent, tail)
        } else {
            None
        }
    } else if head == "*" {
        for child in curr.hierarchy.children(curr.entity()) {
            let child = curr.rebind_any(child);
            if let Some(result) = find_by_path(child, tail) {
                return Some(result);
            }
        }
        return None;
    } else if let Some(child) = curr
        .hierarchy
        .children(curr.entity())
        .map(|child| curr.rebind_any(child))
        .find(|part| part.name().is_some_and(|name| name == head))
    {
        find_by_path(child, tail)
    } else {
        None
    }
}

pub struct ObjectRef<'w, 's, 'a, T: Kind = Any>(EntityRef<'a>, Object<'w, 's, 'a, T>);

impl<T: Component> Deref for ObjectRef<'_, '_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.get::<T>().unwrap()
    }
}

impl<'w, 's, 'a, T: Kind> ObjectRef<'w, 's, 'a, T> {
    pub fn get<U: Component>(&self) -> Option<&U> {
        self.0.get::<U>()
    }

    pub fn contains<U: Component>(&self) -> bool {
        self.0.contains::<U>()
    }

    /// Creates a new [`ObjectRef<T>`] from an [`ObjectRef<Any>`].
    ///
    /// This is semantically equivalent to an unsafe downcast.
    ///
    /// # Safety
    /// Assumes `base` is of [`Kind`] `T`.
    pub unsafe fn from_base_unchecked(base: ObjectRef<'w, 's, 'a>) -> Self {
        Self(base.0, Object::from_base_unchecked(base.1))
    }

    pub fn find_by_path(&self, path: impl AsRef<str>) -> Option<ObjectRef<'w, 's, 'a>> {
        self.1
            .find_by_path(path)
            .map(|object| ObjectRef(self.0, object))
    }
}

impl<T: Kind> Clone for ObjectRef<'_, '_, '_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Kind> Copy for ObjectRef<'_, '_, '_, T> {}

impl<T: Kind> From<ObjectRef<'_, '_, '_, T>> for Entity {
    fn from(object: ObjectRef<'_, '_, '_, T>) -> Self {
        object.entity()
    }
}

impl<T: Kind> From<ObjectRef<'_, '_, '_, T>> for Instance<T> {
    fn from(object: ObjectRef<'_, '_, '_, T>) -> Self {
        object.instance()
    }
}

impl<'w, 's, 'a, T: Kind> From<ObjectRef<'w, 's, 'a, T>> for Object<'w, 's, 'a, T> {
    fn from(object: ObjectRef<'w, 's, 'a, T>) -> Self {
        object.1
    }
}

impl<'w, 's, 'a, T: Kind> From<&ObjectRef<'w, 's, 'a, T>> for Object<'w, 's, 'a, T> {
    fn from(object: &ObjectRef<'w, 's, 'a, T>) -> Self {
        object.1
    }
}

impl<T: Kind> PartialEq for ObjectRef<'_, '_, '_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<T: Kind> Eq for ObjectRef<'_, '_, '_, T> {}

impl<T: Kind> fmt::Debug for ObjectRef<'_, '_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::{ecs::system::RunSystemOnce, prelude::*};

    #[test]
    fn find_by_path() {
        let mut w = World::new();

        //     A
        //    /
        //   B
        //  / \
        // C   D

        let (a, b, c, d) = w.run_system_once(|mut commands: Commands| {
            let a = commands.spawn(Name::new("A")).id();
            let b = commands.spawn(Name::new("B")).id();
            let c = commands.spawn(Name::new("C")).id();
            let d = commands.spawn(Name::new("D")).id();

            commands.entity(a).push_children(&[b]);
            commands.entity(b).push_children(&[c, d]);

            (a, b, c, d)
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path("").unwrap().entity();
            assert_eq!(a, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path(".").unwrap().entity();
            assert_eq!(a, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path("B").unwrap().entity();
            assert_eq!(b, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects
                .get(a)
                .unwrap()
                .find_by_path("B/C")
                .unwrap()
                .entity();
            assert_eq!(c, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects
                .get(a)
                .unwrap()
                .find_by_path("B/D")
                .unwrap()
                .entity();
            assert_eq!(d, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects
                .get(a)
                .unwrap()
                .find_by_path("B/*")
                .unwrap()
                .entity();
            assert_eq!(c, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects
                .get(a)
                .unwrap()
                .find_by_path("*/D")
                .unwrap()
                .entity();
            assert_eq!(d, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects
                .get(a)
                .unwrap()
                .find_by_path("*/*")
                .unwrap()
                .entity();
            assert_eq!(c, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(b).unwrap().find_by_path("..").unwrap().entity();
            assert_eq!(a, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(c).unwrap().find_by_path("..").unwrap().entity();
            assert_eq!(b, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects
                .get(c)
                .unwrap()
                .find_by_path("../D")
                .unwrap()
                .entity();
            assert_eq!(d, x);
        });

        w.run_system_once(move |objects: Objects| {
            let x = objects
                .get(c)
                .unwrap()
                .find_by_path("../C")
                .unwrap()
                .entity();
            assert_eq!(c, x);
        });
    }

    #[test]
    fn object_ref() {
        #[derive(Component)]
        struct T;

        let mut w = World::new();
        let entity = w.spawn(T).id();

        assert!(
            w.run_system_once(move |world: &World, objects: Objects<T>| {
                objects
                    .get_single_ref(world.entity(entity))
                    .unwrap()
                    .contains::<T>()
            })
        );
    }
}
