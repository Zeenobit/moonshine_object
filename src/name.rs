use bevy_ecs::name::Name;
use moonshine_kind::prelude::*;

use crate::{Object, ObjectRef, ObjectWorldRef};

/// [`Object`] methods related to naming.
///
/// These methods are available to any [`Object<T>`] or [`ObjectRef<T>`] type.
pub trait ObjectName {
    /// Returns the [`Name`] of this object.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use moonshine_object::prelude::*;
    ///
    /// let mut app = App::new();
    /// // ...
    /// app.add_systems(Update, print_names);
    ///
    /// fn print_names(objects: Objects) {
    ///     for object in objects.iter() {
    ///         let entity = object.entity();
    ///         let name = object.name().unwrap_or("Unnamed");
    ///         println!("Entity {entity}, Name = {name}");
    ///     }
    /// }
    /// ```
    ///
    /// [`Name`]:bevy_ecs::prelude::Name
    fn name(&self) -> Option<&str>;
}

impl<T: Kind> ObjectName for Object<'_, '_, '_, T> {
    fn name(&self) -> Option<&str> {
        self.nametags
            .get(self.entity())
            .ok()
            .map(|(name, _tags)| name.map(|name| name.as_str()))
            .flatten()
    }
}

impl<T: Kind> ObjectName for ObjectRef<'_, '_, '_, T> {
    fn name(&self) -> Option<&str> {
        self.1.name()
    }
}

impl<T: Kind> ObjectName for ObjectWorldRef<'_, T> {
    fn name(&self) -> Option<&str> {
        self.world
            .get::<Name>(self.entity())
            .map(|name| name.as_str())
    }
}
