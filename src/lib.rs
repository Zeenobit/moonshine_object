use std::fmt;

use bevy_core::Name;
use bevy_ecs::{
    prelude::*,
    query::{QueryData, QueryEntityError, QueryFilter, QueryItem},
    system::SystemParam,
};
use moonshine_kind::{prelude::*, Any};
use moonshine_util::hierarchy::HierarchyQuery;

pub mod prelude {
    pub use super::{safe_object_cast, AsObjectBase, CastObjectInto, Object, Objects};
}

#[derive(SystemParam)]
pub struct Objects<'w, 's, T = Any, F = ()>
where
    T: Kind,
    F: 'static + QueryFilter,
{
    pub instances: Query<'w, 's, Instance<T>, F>,
    pub hierarchy: HierarchyQuery<'w, 's>,
    pub name: Query<'w, 's, &'static Name>,
}

impl<'w, 's, T, F> Objects<'w, 's, T, F>
where
    T: Kind,
    F: 'static + QueryFilter,
{
    pub fn iter(&self) -> impl Iterator<Item = Object<'w, 's, '_, T>> {
        self.instances.iter().map(|instance| Object {
            instance,
            hierarchy: &self.hierarchy,
            name: &self.name,
        })
    }

    pub fn get(&self, entity: Entity) -> Result<Object<'w, 's, '_, T>, QueryEntityError> {
        self.instances.get(entity).map(|instance| Object {
            instance,
            hierarchy: &self.hierarchy,
            name: &self.name,
        })
    }

    #[must_use]
    pub fn instance(&self, instance: impl Into<Instance<T>>) -> Object<'w, 's, '_, T> {
        self.get(instance.into().entity()).unwrap()
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        bundle: impl KindBundle<Kind = T>,
    ) -> Object<'w, 's, '_, T> {
        let instance = commands.spawn_instance(bundle);
        Object {
            instance: instance.into(),
            hierarchy: &self.hierarchy,
            name: &self.name,
        }
    }
}

/// Represents an [`Entity`] of [`Kind`] `T` with [`HierarchyQuery`] and [`Name`] information.
pub struct Object<'w, 's, 'a, T: Kind = Any> {
    instance: Instance<T>,
    hierarchy: &'a HierarchyQuery<'w, 's>,
    name: &'a Query<'w, 's, &'static Name>,
}

impl<'w, 's, 'a, T: Kind> Object<'w, 's, 'a, T> {
    /// # Safety
    /// Assumes `base` is of kind `T`.
    #[must_use]
    pub unsafe fn from_base_unchecked(base: Object<'w, 's, 'a>) -> Self {
        Self {
            instance: base.instance.cast_into_unchecked(),
            hierarchy: base.hierarchy,
            name: base.name,
        }
    }

    #[must_use]
    pub fn instance(&self) -> Instance<T> {
        self.instance
    }

    #[must_use]
    pub fn entity(&self) -> Entity {
        self.instance.entity()
    }

    #[must_use]
    pub fn name_or_default(&self) -> &str {
        self.name
            .get(self.entity())
            .map(|name| name.as_str())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn is_root(&self) -> bool {
        self.hierarchy.is_root(self.entity())
    }

    #[must_use]
    pub fn is_parent(&self) -> bool {
        self.has_children()
    }

    #[must_use]
    pub fn is_child(&self) -> bool {
        self.parent().is_some()
    }

    #[must_use]
    pub fn has_children(&self) -> bool {
        self.children().peekable().peek().is_some()
    }

    #[must_use]
    pub fn is_descendant_of(&self, entity: impl Into<Entity>) -> bool {
        self.hierarchy
            .is_descendant_of(self.entity(), entity.into())
    }

    #[must_use]
    pub fn find_by_path(&self, path: &str) -> Option<Object<'w, 's, 'a>> {
        let tail: Vec<&str> = path.split('/').collect();
        find_by_path(self.as_base(), &tail)
    }

    #[must_use]
    pub fn root(&self) -> Object<'w, 's, 'a> {
        let entity = self.hierarchy.root(self.entity());
        self.rebind_as_base(entity)
    }

    #[must_use]
    pub fn parent(&self) -> Option<Object<'w, 's, 'a>> {
        self.hierarchy
            .parent(self.entity())
            .map(|entity| self.rebind_as_base(entity))
    }

    pub fn children(&self) -> impl Iterator<Item = Object<'w, 's, 'a>> + '_ {
        self.hierarchy
            .children(self.entity())
            .map(|entity| self.rebind_as_base(entity))
    }

    pub fn ancestors(&self) -> impl Iterator<Item = Object<'w, 's, 'a>> + '_ {
        self.hierarchy
            .ancestors(self.entity())
            .map(|entity| self.rebind_as_base(entity))
    }

    pub fn query_ancestors<Q: QueryData>(
        &'a self,
        query: &'a Query<'w, 's, Q>,
    ) -> impl Iterator<Item = QueryItem<'_, Q::ReadOnly>> + 'a {
        self.ancestors().filter_map(move |object| {
            let entity = object.entity();
            query.get(entity).ok()
        })
    }

    pub fn descendants(&self) -> impl Iterator<Item = Object<'w, 's, 'a>> + '_ {
        self.hierarchy
            .descendants(self.entity())
            .map(|entity| self.rebind_as_base(entity))
    }

    #[must_use]
    pub fn as_base(&self) -> Object<'w, 's, 'a> {
        Object {
            instance: self.instance.cast_into(),
            hierarchy: self.hierarchy,
            name: self.name,
        }
    }

    pub fn rebind(&self, instance: Instance<T>) -> Object<'w, 's, 'a, T> {
        Object {
            instance,
            hierarchy: self.hierarchy,
            name: self.name,
        }
    }

    #[must_use]
    pub fn rebind_as_base(&self, entity: Entity) -> Object<'w, 's, 'a> {
        Object {
            instance: entity.into(),
            hierarchy: self.hierarchy,
            name: self.name,
        }
    }

    #[must_use]
    pub fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Object<'w, 's, 'a, U>
    where
        T: CastObjectInto<'w, 's, 'a, U>,
    {
        Object {
            instance,
            hierarchy: self.hierarchy,
            name: self.name,
        }
    }

    /// # Safety
    /// Assumes `T` is also a valid instance of `U`.
    #[must_use]
    pub unsafe fn rebind_as_unchecked<U: Kind>(
        &self,
        instance: Instance<U>,
    ) -> Object<'w, 's, 'a, U> {
        Object {
            instance,
            hierarchy: self.hierarchy,
            name: self.name,
        }
    }

    pub fn cast_into<U: Kind>(self) -> Object<'w, 's, 'a, U>
    where
        T: CastObjectInto<'w, 's, 'a, U>,
    {
        T::cast_object_into(self)
    }

    /// # Safety
    /// Assumes `T` is also a valid instance of `U`.
    pub unsafe fn cast_into_unchecked<U: Kind>(self) -> Object<'w, 's, 'a, U> {
        Object {
            instance: self.instance.cast_into_unchecked(),
            hierarchy: self.hierarchy,
            name: self.name,
        }
    }
}

impl<'w, 's, 'a, T: Component> Object<'w, 's, 'a, T> {
    #[must_use]
    pub fn from_base(world: &World, object: Object<'w, 's, 'a>) -> Option<Object<'w, 's, 'a, T>> {
        let entity = world.entity(object.entity());
        let instance = Instance::<T>::from_entity(entity)?;
        // SAFE: Entity was just checked to a valid instance of T.
        Some(unsafe { object.rebind_as_unchecked(instance) })
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
        f.debug_tuple(&T::debug_name())
            .field(&self.entity())
            .field(&self.name_or_default())
            .finish()
    }
}

pub trait AsObjectBase {
    fn as_base(&self) -> Object<'_, '_, '_>;
}

impl<T: Kind> AsObjectBase for Object<'_, '_, '_, T> {
    fn as_base(&self) -> Object<'_, '_, '_> {
        self.as_base()
    }
}

/// # Safety
/// Prefer to use the `safe_object_cast!` macro instead.
pub unsafe trait CastObjectInto<'w, 's, 'a, T: Kind>: Kind {
    fn cast_object_into(object: Object<'w, 's, 'a, Self>) -> Object<'w, 's, 'a, T>;
}

unsafe impl<'w, 's, 'a, T: Kind> CastObjectInto<'w, 's, 'a, Any> for T {
    fn cast_object_into(object: Object<'w, 's, 'a, Self>) -> Object<'w, 's, 'a, Any> {
        object.as_base()
    }
}

#[macro_export]
macro_rules! safe_object_cast {
    ($t:ty => $u:ty) => {
        unsafe impl<'w, 's, 'a> $crate::CastObjectInto<'w, 's, 'a, $u> for $t {
            fn cast_object_into(object: Object<'w, 's, 'a, Self>) -> Object<'w, 's, 'a, $u> {
                // SAFE: Because we said so!
                unsafe { object.cast_into_unchecked() }
            }
        }
    };
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
            .map(|parent| curr.rebind_as_base(parent))
        {
            find_by_path(parent, tail)
        } else {
            None
        }
    } else if head == "*" {
        if let Some(child) = curr
            .hierarchy
            .children(curr.entity())
            .map(|child| curr.rebind_as_base(child))
            .next()
        {
            find_by_path(child, tail)
        } else {
            None
        }
    } else if let Some(child) = curr
        .hierarchy
        .children(curr.entity())
        .map(|child| curr.rebind_as_base(child))
        .find(|part| part.name_or_default() == head)
    {
        find_by_path(child, tail)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    #[test]
    fn find_by_path_self() {
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
}
