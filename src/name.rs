use moonshine_kind::prelude::*;

use crate::{Object, ObjectRef};

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
    /// [`Name`]: https://docs.rs/bevy/latest/bevy/core/struct.Name.html
    fn name(&self) -> Option<&str>;
}

impl<T: Kind> ObjectName for Object<'_, '_, '_, T> {
    fn name(&self) -> Option<&str> {
        self.name.get(self.entity()).ok().map(|name| name.as_str())
    }
}

impl<T: Kind> ObjectName for ObjectRef<'_, '_, '_, T> {
    fn name(&self) -> Option<&str> {
        self.1.name()
    }
}
