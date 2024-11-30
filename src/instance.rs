use bevy_ecs::prelude::*;
use moonshine_kind::{prelude::*, Any};

use crate::{Object, ObjectRef};

pub trait ObjectInstance<T: Kind = Any> {
    /// Returns the [`Instance`] of this object.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use moonshine_object::prelude::*;
    ///
    /// let mut app = App::new();
    /// // ...
    /// app.add_systems(Update, print_instances);
    ///
    /// fn print_instances(objects: Objects) {
    ///     for object in objects.iter() {
    ///         println!("{:?}", object.instance());
    ///     }
    /// }
    /// ```
    fn instance(&self) -> Instance<T>;

    /// Returns the [`Entity`] of this object.
    fn entity(&self) -> Entity {
        self.instance().entity()
    }
}

impl<T: Kind> ObjectInstance<T> for Object<'_, '_, '_, T> {
    fn instance(&self) -> Instance<T> {
        self.instance
    }
}

impl<T: Kind> ObjectInstance<T> for ObjectRef<'_, '_, '_, T> {
    fn instance(&self) -> Instance<T> {
        self.1.instance()
    }
}
