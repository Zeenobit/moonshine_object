use moonshine_kind::{prelude::*, Any, CastInto};

use crate::{Object, ObjectInstance, ObjectRebind, ObjectRef};

pub trait ObjectCast<T: Kind = Any>: ObjectInstance<T> + ObjectRebind<T> + Sized {
    fn cast_into<U: Kind>(self) -> Self::Rebind<U>
    where
        T: CastInto<U>,
    {
        self.rebind_as(self.instance().cast_into())
    }

    fn cast_into_any(self) -> Self::Rebind<Any> {
        self.rebind_as(self.instance().cast_into_any())
    }

    /// # Safety
    ///
    /// TODO
    unsafe fn cast_into_unchecked<U: Kind>(self) -> Self::Rebind<U> {
        self.rebind_as(self.instance().cast_into_unchecked())
    }
}

impl<T: Kind> ObjectCast<T> for Object<'_, '_, '_, T> {}

impl<T: Kind> ObjectCast<T> for ObjectRef<'_, '_, '_, T> {}
