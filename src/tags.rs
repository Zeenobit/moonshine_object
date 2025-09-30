use moonshine_kind::prelude::*;
use moonshine_tag::Tags;

use crate::{Object, ObjectRef, ObjectWorldRef};

/// [`Object`] methods related to accessing [`Tags`].
pub trait ObjectTags {
    /// Returns the [`Tags`] of this [`Object`].
    ///
    /// For convenience, if the object has no tags, [`Tags::static_empty`] is returned instead.
    fn tags(&self) -> &Tags;
}

impl<T: Kind> ObjectTags for Object<'_, '_, '_, T> {
    fn tags(&self) -> &Tags {
        self.nametags
            .get(self.entity())
            .ok()
            .and_then(|(_name, tags)| tags)
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
