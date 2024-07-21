use moonshine_kind::prelude::*;

use crate::{Object, ObjectInstance, ObjectRef};

pub trait ObjectName {
    /// Returns the [`Name`] of the object.
    ///
    /// # Example
    ///
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
    ///
    /// # bevy::ecs::system::assert_is_system(print_names);
    /// ```
    ///
    /// [`Name`]: https://docs.rs/bevy/latest/bevy/core/struct.Name.html
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
