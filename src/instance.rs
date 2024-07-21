use bevy_ecs::prelude::*;
use moonshine_kind::{prelude::*, Any};

use crate::{Object, ObjectRef};

pub trait ObjectInstance<T: Kind = Any> {
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
