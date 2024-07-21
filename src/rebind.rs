use bevy_ecs::prelude::*;
use moonshine_kind::{prelude::*, Any};

use crate::{Object, ObjectHierarchy, ObjectInstance, ObjectRef};

pub trait ObjectRebind<T: Kind = Any>: ObjectInstance<T> {
    type Rebind<U: Kind>: ObjectHierarchy<U>;

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
