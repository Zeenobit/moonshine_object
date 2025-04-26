#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod prelude {
    //! Prelude module to import all necessary traits and types for working with objects.

    pub use super::{Object, ObjectRef, Objects, RootObjects};
    pub use super::{ObjectHierarchy, ObjectName, ObjectRebind};
}

use std::fmt;
use std::ops::Deref;

use bevy_ecs::prelude::*;
use bevy_ecs::query::{QueryEntityError, QueryFilter, QuerySingleError};
use bevy_ecs::system::SystemParam;
use moonshine_kind::prelude::*;
use moonshine_util::hierarchy::HierarchyQuery;

pub use moonshine_kind::{Any, CastInto, Kind};

/// A [`SystemParam`] similar to [`Query`] which provides [`Object<T>`] access for its items.
#[derive(SystemParam)]
pub struct Objects<'w, 's, T = Any, F = ()>
where
    T: Kind,
    F: 'static + QueryFilter,
{
    /// [`Query`] used to filter instances of the given [`Kind`] `T`.
    pub instance: Query<'w, 's, Instance<T>, F>,
    /// [`HierarchyQuery`] used to traverse the object hierarchy.
    pub hierarchy: HierarchyQuery<'w, 's>,
    /// [`Query`] to get names of objects, mainly used for for hierarchy traversal by path and debugging.
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

    /// Iterates over all [`Object`]s of [`Kind`] `T` which match the [`QueryFilter`] `F`.
    #[deprecated(since = "0.2.1", note = "use `RootObjects` instead")]
    pub fn iter_root(&self) -> impl Iterator<Item = Object<'w, 's, '_, T>> {
        self.iter().filter(|object| object.is_root())
    }

    /// Returns true if the given [`Entity`] is a valid [`Object<T>`].
    pub fn contains(&self, entity: Entity) -> bool {
        self.instance.contains(entity)
    }

    #[deprecated(since = "0.2.1", note = "use `RootObjects` instead")]
    #[doc(hidden)]
    pub fn contains_root(&self, entity: Entity) -> bool {
        self.get(entity).is_ok_and(|object| object.is_root())
    }

    /// Returns an iterator over all [`ObjectRef`] instances of [`Kind`] `T` which match the [`QueryFilter`] `F`.
    pub fn iter_ref<'a>(
        &'a self,
        world: &'a World,
    ) -> impl Iterator<Item = ObjectRef<'w, 's, 'a, T>> {
        self.iter()
            .map(|object: Object<T>| ObjectRef(world.entity(object.entity()), object))
    }

    #[deprecated(since = "0.2.1", note = "use `RootObjects` instead")]
    #[doc(hidden)]
    pub fn iter_root_ref<'a>(
        &'a self,
        world: &'a World,
    ) -> impl Iterator<Item = ObjectRef<'w, 's, 'a, T>> {
        self.iter()
            .map(|object: Object<T>| ObjectRef(world.entity(object.entity()), object))
    }

    /// Returns an [`Object<T>`] from an [`Entity`], if it matches [`QueryFilter`] `F`.
    pub fn get(&self, entity: Entity) -> Result<Object<'w, 's, '_, T>, QueryEntityError> {
        self.instance.get(entity).map(|instance| Object {
            instance,
            hierarchy: &self.hierarchy,
            name: &self.name,
        })
    }

    #[deprecated(since = "0.2.1", note = "use `RootObjects` instead")]
    #[doc(hidden)]
    pub fn get_root(&self, entity: Entity) -> Option<Object<'w, 's, '_, T>> {
        self.get(entity)
            .ok()
            .and_then(|object| if object.is_root() { Some(object) } else { None })
    }

    /// Returns an [`ObjectRef<T>`] from an [`EntityRef`], if it matches [`QueryFilter`] `F`.
    pub fn get_ref<'a>(&'a self, entity: EntityRef<'a>) -> Option<ObjectRef<'w, 's, 'a, T>> {
        Some(ObjectRef(entity, self.get(entity.id()).ok()?))
    }

    /// Returns an [`Object<T>`], if it exists as a single instance.
    pub fn get_single(&self) -> Result<Object<'w, 's, '_, T>, QuerySingleError> {
        self.instance.single().map(|instance| Object {
            instance,
            hierarchy: &self.hierarchy,
            name: &self.name,
        })
    }

    #[deprecated(since = "0.2.1", note = "use `RootObjects` instead")]
    #[doc(hidden)]
    pub fn get_single_root(&self) -> Result<Object<'w, 's, '_, T>, QuerySingleError> {
        self.get_single().and_then(|object| {
            if object.is_root() {
                Ok(object)
            } else {
                // NOTE: Not the most accurate error data, but the function is deprecated. Will be removed soon.
                Err(QuerySingleError::NoEntities("Object is not root"))
            }
        })
    }

    /// Returns an [`ObjectRef<T>`] from an [`EntityRef`], if it exists as a single instance.
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

/// Ergonomic type alias for all [`Objects`] of [`Kind`] `T` without a parent.
pub type RootObjects<'w, 's, T = Any, F = ()> = Objects<'w, 's, T, (F, Without<ChildOf>)>;

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

    /// Returns the object as an [`Instance<T>`].
    pub fn instance(&self) -> Instance<T> {
        self.instance
    }

    /// Returns the object as an [`Entity`].
    pub fn entity(&self) -> Entity {
        self.instance.entity()
    }
}

impl<'w, 's, 'a, T: Component> Object<'w, 's, 'a, T> {
    /// Creates a new [`Object<T>`] from an [`Object<Any>`] if it is a valid instance of `T`.
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

impl<T: Kind> PartialEq<Instance<T>> for Object<'_, '_, '_, T> {
    fn eq(&self, other: &Instance<T>) -> bool {
        self.instance() == *other
    }
}

impl<T: Kind> PartialEq<Object<'_, '_, '_, T>> for Instance<T> {
    fn eq(&self, other: &Object<T>) -> bool {
        *self == other.instance()
    }
}

impl<T: Kind> PartialEq<Entity> for Object<'_, '_, '_, T> {
    fn eq(&self, other: &Entity) -> bool {
        self.entity() == *other
    }
}

impl<T: Kind> PartialEq<Object<'_, '_, '_, T>> for Entity {
    fn eq(&self, other: &Object<T>) -> bool {
        *self == other.entity()
    }
}

impl<T: Kind> fmt::Debug for Object<'_, '_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{}({:?}, \"{}\")", &T::debug_name(), self.entity(), name)
        } else {
            write!(f, "{}({:?})", &T::debug_name(), self.entity())
        }
    }
}

impl<T: Kind> fmt::Display for Object<'_, '_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{}({}, \"{}\")", &T::debug_name(), self.entity(), name)
        } else {
            write!(f, "{}({})", &T::debug_name(), self.entity())
        }
    }
}

impl<T: Kind> ContainsInstance<T> for Object<'_, '_, '_, T> {
    fn instance(&self) -> Instance<T> {
        self.instance
    }
}

/// Similar to [`EntityRef`] with the benefits of [`Object<T>`].
pub struct ObjectRef<'w, 's, 'a, T: Kind = Any>(EntityRef<'a>, Object<'w, 's, 'a, T>);

impl<'w, 's, 'a, T: Kind> ObjectRef<'w, 's, 'a, T> {
    /// Creates a new [`ObjectRef<T>`] from an [`ObjectRef<Any>`].
    ///
    /// This is semantically equivalent to an unsafe downcast.
    ///
    /// # Safety
    /// Assumes `base` is of [`Kind`] `T`.
    pub unsafe fn from_base_unchecked(base: ObjectRef<'w, 's, 'a>) -> Self {
        Self(base.0, Object::from_base_unchecked(base.1))
    }

    /// See [`EntityRef::get`].
    pub fn get<U: Component>(&self) -> Option<&U> {
        self.0.get::<U>()
    }

    /// See [`EntityRef::contains`].
    pub fn contains<U: Component>(&self) -> bool {
        self.0.contains::<U>()
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

impl<T: Kind> ContainsInstance<T> for ObjectRef<'_, '_, '_, T> {
    fn instance(&self) -> Instance<T> {
        self.1.instance()
    }
}

impl<T: Component> Deref for ObjectRef<'_, '_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.get::<T>().unwrap()
    }
}

impl<T: Kind> fmt::Debug for ObjectRef<'_, '_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{}({:?}, \"{}\")", &T::debug_name(), self.entity(), name)
        } else {
            write!(f, "{}({:?})", &T::debug_name(), self.entity())
        }
    }
}

impl<T: Kind> fmt::Display for ObjectRef<'_, '_, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{}({}, \"{}\")", &T::debug_name(), self.entity(), name)
        } else {
            write!(f, "{}({})", &T::debug_name(), self.entity())
        }
    }
}

mod hierarchy;
mod name;
mod rebind;

pub use hierarchy::*;
pub use name::*;
pub use rebind::*;

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn find_by_path() {
        let mut w = World::new();

        //     A
        //    /
        //   B
        //  / \
        // C   D

        let (a, b, c, d) = w
            .run_system_once(|mut commands: Commands| {
                let a = commands.spawn(Name::new("A")).id();
                let b = commands.spawn(Name::new("B")).id();
                let c = commands.spawn(Name::new("C")).id();
                let d = commands.spawn(Name::new("D")).id();

                commands.entity(a).add_children(&[b]);
                commands.entity(b).add_children(&[c, d]);

                (a, b, c, d)
            })
            .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path("").unwrap();
            assert_eq!(a, x);
            assert_eq!(x.path(), "A");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path(".").unwrap();
            assert_eq!(a, x);
            assert_eq!(x.path(), "A");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path("B").unwrap();
            assert_eq!(b, x);
            assert_eq!(x.path(), "A/B");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path("B/C").unwrap();
            assert_eq!(c, x);
            assert_eq!(x.path(), "A/B/C");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path("B/D").unwrap();
            assert_eq!(d, x);
            assert_eq!(x.path(), "A/B/D");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path("B/*").unwrap();
            assert_eq!(c, x);
            assert_eq!(x.path(), "A/B/C");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path("*/D").unwrap();
            assert_eq!(d, x);
            assert_eq!(x.path(), "A/B/D");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(a).unwrap().find_by_path("*/*").unwrap();
            assert_eq!(c, x);
            assert_eq!(x.path(), "A/B/C");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(b).unwrap().find_by_path("..").unwrap();
            assert_eq!(a, x);
            assert_eq!(x.path(), "A");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(c).unwrap().find_by_path("..").unwrap();
            assert_eq!(b, x);
            assert_eq!(x.path(), "A/B");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(c).unwrap().find_by_path("../D").unwrap();
            assert_eq!(d, x);
            assert_eq!(x.path(), "A/B/D");
        })
        .unwrap();

        w.run_system_once(move |objects: Objects| {
            let x = objects.get(c).unwrap().find_by_path("../C").unwrap();
            assert_eq!(c, x);
            assert_eq!(x.path(), "A/B/C");
        })
        .unwrap();
    }

    #[test]
    fn object_ref() {
        #[derive(Component)]
        struct T;

        let mut w = World::new();
        let entity = w.spawn(T).id();

        assert!(w
            .run_system_once(move |world: &World, objects: Objects<T>| {
                objects
                    .get_single_ref(world.entity(entity))
                    .unwrap()
                    .contains::<T>()
            })
            .unwrap());
    }

    #[test]
    fn root_objects() {
        #[derive(Component)]
        struct T;

        //     A
        //    /
        //   B
        //  / \
        // C   D

        let mut w = World::new();
        let root = w
            .spawn(T) /* A */
            .with_children(|children| {
                children.spawn(T /* B */).with_children(|children| {
                    children.spawn(T /* C */);
                    children.spawn(T /* D */);
                });
            })
            .id();

        assert!(w
            .run_system_once(move |objects: RootObjects<T>| {
                assert_eq!(objects.iter().count(), 1);
                assert!(objects.contains(root));
                assert!(objects.get_single().is_ok());
                true
            })
            .unwrap());
    }
}
