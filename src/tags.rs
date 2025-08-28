use moonshine_kind::prelude::*;
use moonshine_tag::Tags;

use crate::{Object, ObjectRef, ObjectWorldRef};

pub trait ObjectTags {
    fn tags(&self) -> &Tags;
}

impl<T: Kind> ObjectTags for Object<'_, '_, '_, T> {
    fn tags(&self) -> &Tags {
        self.nametags
            .get(self.entity())
            .ok()
            .map(|(_name, tags)| tags)
            .flatten()
            .unwrap_or(Tags::static_empty())
    }
}

impl<T: Kind> ObjectTags for ObjectRef<'_, '_, '_, T> {
    fn tags(&self) -> &Tags {
        self.1.tags()
    }
}

impl<T: Kind> ObjectTags for ObjectWorldRef<'_, T> {
    fn tags(&self) -> &Tags {
        self.world
            .get(self.entity())
            .unwrap_or(Tags::static_empty())
    }
}
