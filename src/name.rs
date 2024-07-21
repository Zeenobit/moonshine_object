use moonshine_kind::prelude::*;

use crate::{Object, ObjectInstance, ObjectRef};

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
