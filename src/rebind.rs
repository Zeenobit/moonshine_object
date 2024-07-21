use bevy_ecs::prelude::*;
use moonshine_kind::{prelude::*, Any, CastInto};

use crate::{Object, ObjectHierarchy, ObjectInstance, ObjectRef};

pub trait ObjectRebind<T: Kind = Any>: ObjectInstance<T> + Sized {
    type Rebind<U: Kind>: ObjectHierarchy<U>;

    /// Rebinds this object to an [`Instance`] of another [`Kind`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] and an [`Instance<U>`]
    /// but you want an [`Object<U>`].
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
    /// fn find_worms(apples: Objects<Apple>, query: Query<&Apple>) {
    ///     for object in apples.iter() {
    ///         let apple = query.get(object.entity()).unwrap();
    ///         for worm in apple.worms.iter() {
    ///             handle_worm(object.rebind_as(*worm));
    ///         }
    ///     }
    /// }
    ///
    /// fn handle_worm(worm: Object<Worm>) {
    ///     println!("{:?} found! Gross!", worm);
    /// }
    /// ```
    fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U>;

    /// Rebinds this object to another [`Instance`] of the same [`Kind`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] and another [`Instance<T>`]
    /// but you want another [`Object<T>`].
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
    ///             greet_friend(object.rebind(*friend));
    ///         }
    ///     }
    /// }
    ///
    /// fn greet_friend(friend: Object<Person>) {
    ///     println!("Hello {:?}!", friend);
    /// }
    /// ```
    fn rebind(&self, instance: Instance<T>) -> Self::Rebind<T> {
        self.rebind_as(instance)
    }

    /// Rebinds this object to another [`Entity`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] but you want an [`Object`] for a different [`Entity`].
    fn rebind_any(&self, entity: Entity) -> Self::Rebind<Any> {
        self.rebind_as(Instance::from(entity))
    }

    /// Casts this object into another of a related [`Kind`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] but you want an [`Object<U>`]
    /// where [`Kind`] `T` is safely convertible to `U`.
    ///
    /// See [`kind!`] for more information on kind conversion.
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
    /// kind!(Apple is Fruit);
    /// kind!(Orange is Fruit);
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
        self.rebind_as(self.instance().cast_into())
    }

    /// Casts this object into an [`Object<Any>`].
    ///
    /// # Usage
    ///
    /// This is useful when you have an [`Object<T>`] but you want an [`Object<Any>`].
    ///
    /// All objects of any [`Kind`] can be cast into [`Object<Any>`].
    fn cast_into_any(self) -> Self::Rebind<Any> {
        self.rebind_as(self.instance().cast_into_any())
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
}

impl<'w, 's, 'a, T: Kind> ObjectRebind<T> for Object<'w, 's, 'a, T> {
    type Rebind<U: Kind> = Object<'w, 's, 'a, U>;

    fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U> {
        Object {
            instance,
            hierarchy: self.hierarchy,
            name: self.name,
        }
    }
}

impl<'w, 's, 'a, T: Kind> ObjectRebind<T> for ObjectRef<'w, 's, 'a, T> {
    type Rebind<U: Kind> = ObjectRef<'w, 's, 'a, U>;

    fn rebind_as<U: Kind>(&self, instance: Instance<U>) -> Self::Rebind<U> {
        ObjectRef(self.0, self.1.rebind_as(instance))
    }
}
