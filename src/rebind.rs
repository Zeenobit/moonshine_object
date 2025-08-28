use bevy_ecs::prelude::*;
use moonshine_kind::{prelude::*, Any, CastInto};

use crate::{Object, ObjectHierarchy, ObjectRef, ObjectWorldRef};

/// [`Object`] methods related to rebinding and casting.
///
/// These methods are available to any [`Object<T>`] or [`ObjectRef<T>`] type.
pub trait ObjectRebind<T: Kind = Any>: ContainsInstance<T> + Sized {
    #[doc(hidden)]
    type Rebind<U: Kind>: ObjectHierarchy<U>;

    /// Rebinds this object to an [`Instance`] of another [`Kind`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] and an [`Instance<U>`]
    /// but you want an [`Object<U>`].
    ///
    /// # Safety
    /// This method assumes the given instance is valid.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use moonshine_object::prelude::*;
    /// # use moonshine_kind::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Apple {
    ///     worms: Vec<Instance<Worm>>,
    /// }
    ///
    /// #[derive(Component)]
    /// struct Worm;
    ///
    /// let mut app = App::new();
    /// // ...
    /// app.add_systems(Update, find_worms);
    ///
    /// fn find_worms(apples: Objects<Apple>, query: Query<&Apple>, worms: Query<&Worm>) {
    ///     for object in apples.iter() {
    ///         let apple = query.get(object.entity()).unwrap();
    ///         for worm in apple.worms.iter() {
    ///             if worms.contains(*worm) {
    ///                 // SAFE: We just checked that the worm exists
    ///                 handle_worm(unsafe { object.rebind_as(*worm) });
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// fn handle_worm(worm: Object<Worm>) {
    ///     println!("{:?} found! Gross!", worm);
    /// }
    /// ```
    unsafe fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U>;

    /// Rebinds this object to another [`Instance`] of the same [`Kind`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] and another [`Instance<T>`]
    /// but you want another [`Object<T>`].
    ///
    /// # Safety
    ///
    /// This method assumes the given instance is valid.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use moonshine_object::prelude::*;
    /// # use moonshine_kind::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Person {
    ///     friends: Vec<Instance<Person>>,
    /// }
    ///
    /// let mut app = App::new();
    /// // ...
    /// app.add_systems(Update, update_friends);
    ///
    /// fn update_friends(people: Objects<Person>, query: Query<&Person>) {
    ///     for object in people.iter() {
    ///         let person = query.get(object.entity()).unwrap();
    ///         for friend in person.friends.iter() {
    ///             if !people.contains(*friend) {
    ///                 continue;
    ///             }
    ///             // SAFE: We just checked that the friend exists
    ///             greet_friend(unsafe { object.rebind(*friend) });
    ///         }
    ///     }
    /// }
    ///
    /// fn greet_friend(friend: Object<Person>) {
    ///     println!("Hello {:?}!", friend);
    /// }
    /// ```
    unsafe fn rebind(&self, instance: Instance<T>) -> Self::Rebind<T> {
        self.rebind_as(instance)
    }

    /// Rebinds this object to another [`Entity`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] but you want an [`Object`] for a different [`Entity`].
    ///
    /// # Safety
    ///
    /// This method assumes the given entity is valid.
    unsafe fn rebind_any(&self, entity: Entity) -> Self::Rebind<Any> {
        self.rebind_as(Instance::from(entity))
    }

    /// Casts this object into another of a related [`Kind`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] but you want an [`Object<U>`]
    /// where [`Kind`] `T` is safely convertible to `U`.
    ///
    /// See [`CastInto`] for more information on kind conversion.
    ///
    /// # Example
    /// ```
    /// # use bevy::prelude::*;
    /// # use moonshine_object::prelude::*;
    /// # use moonshine_kind::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Apple;
    ///
    /// #[derive(Component)]
    /// struct Orange;
    ///
    /// struct Fruit;
    ///
    /// impl Kind for Fruit {
    ///     // Apples and Oranges are fruits.
    ///     type Filter = Or<(With<Apple>, With<Orange>)>;
    /// }
    ///
    /// // Define related kinds:
    /// impl CastInto<Fruit> for Apple {}
    /// impl CastInto<Fruit> for Orange {}
    ///
    /// let mut app = App::new();
    /// // ...
    /// app.add_systems(Update, (eat_apples, eat_oranges));
    ///
    /// fn eat_apples(apples: Objects<Apple>) {
    ///     for apple in apples.iter() {
    ///         eat_fruit(apple.cast_into());
    ///         println!("Crunchy!")
    ///     }
    /// }
    ///
    /// fn eat_oranges(oranges: Objects<Orange>) {
    ///     for orange in oranges.iter() {
    ///         eat_fruit(orange.cast_into());
    ///         println!("Juicy!")
    ///     }
    /// }
    ///
    /// fn eat_fruit(fruit: Object<Fruit>) {
    ///     println!("{:?} is eaten!", fruit);
    /// }
    /// ```
    fn cast_into<U: Kind>(self) -> Self::Rebind<U>
    where
        T: CastInto<U>,
    {
        // SAFE: T is safely convertible to U, and it is the same entity
        unsafe { self.rebind_as(self.instance().cast_into()) }
    }

    /// Casts this object into an [`Object<Any>`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] but you want an [`Object<Any>`].
    ///
    /// All objects of any [`Kind`] can be cast into [`Object<Any>`].
    fn cast_into_any(self) -> Self::Rebind<Any> {
        // SAFE: T is safely convertible to Any, and it is the same entity
        unsafe { self.rebind_as(self.instance().cast_into_any()) }
    }

    /// Casts this object into another of a different [`Kind`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] but you want an [`Object<U>`] and
    /// you can guarantee that [`Kind`] `T` is safely convertible to `U`.
    ///
    /// # Safety
    ///
    /// It is assumed that [`Kind`] `T` is safely convertible to `U`.
    unsafe fn cast_into_unchecked<U: Kind>(self) -> Self::Rebind<U> {
        self.rebind_as(self.instance().cast_into_unchecked())
    }

    /// Returns this object as an [`Object<Any>`].
    fn as_any(&self) -> Self::Rebind<Any> {
        // SAFE: I'm valid if I'm valid.
        unsafe { self.rebind_any(self.entity()) }
    }
}

impl<'w, 's, 'a, T: Kind> ObjectRebind<T> for Object<'w, 's, 'a, T> {
    type Rebind<U: Kind> = Object<'w, 's, 'a, U>;

    unsafe fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U> {
        Object {
            instance,
            hierarchy: self.hierarchy,
            nametags: self.nametags,
        }
    }
}

impl<'w, 's, 'a, T: Kind> ObjectRebind<T> for ObjectRef<'w, 's, 'a, T> {
    type Rebind<U: Kind> = ObjectRef<'w, 's, 'a, U>;

    unsafe fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U> {
        ObjectRef(self.0, self.1.rebind_as(instance))
    }
}

impl<'w, T: Kind> ObjectRebind<T> for ObjectWorldRef<'w, T> {
    type Rebind<U: Kind> = ObjectWorldRef<'w, U>;

    unsafe fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U> {
        ObjectWorldRef {
            instance: instance,
            world: self.world,
        }
    }
}
