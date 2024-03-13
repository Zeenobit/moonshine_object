# 🌴 Moonshine Objects

An extension to [Bevy](https://bevyengine.org) entities to make complex ECS hierarchies more ergonomic.

Entities are nice. Objects are *better*! 😎

## Overview

`Object` provides an ergonomic interface for traversing and querying an entity hierarchy.

It is a wrapper around 3 common queries in Bevy:

```
Query<&Name>
Query<&Parent>
Query<&Children>
```

Any `Entity` can be expressed as an `Object`.

Additionally, an `Object<T: Kind>` provides type safety through [Kinds](https://github.com/Zeenobit/moonshine_kind).

An `Object` is not a query item itself, rather accessible via the `Objects<T>` system parameter:

```rust
#[derive(Component)]
struct Bird;

#[derive(Bundle)]
struct BirdBundle {
    bird: Bird,
    name: Name, // Optional!
}

fn bird_system(birds: Objects<Bird>, mut commands: Commands) {
    for bird in birds.iter() {
        flap_wings(bird, &mut commands);
    }
}
```

Because an `Object` has access to hierarchy and name information, it can provide a set of useful functions, such as:

```rust,ignore
impl<T: Kind> Object<T> {
    fn is_root(&self) -> bool;
    fn is_parent(&self) -> bool;
    fn is_child(&self) -> bool;
    fn is_descendant_of(&self, entity: impl Into<Entity>) -> bool;
    fn find_by_path(&self, path: &str) -> Option<Object>;
    fn root(&self) -> Object;
    fn parent(&self) -> Option<Object>;
    fn children(&self) -> impl Iterator<Item = Object>;
    fn ancestors(&self) -> impl Iterator<Item = Object>;
    // ...
}
```

This makes it very easy to pass this information between your systems and functions:

```rust
fn flap_wings(bird: Object<Bird>, commands: &mut Commands) {
    if let Some(wings) = bird.find_by_path("body/wings") {
        // TODO: Update state of wings
        for wing in wings.children() {
            // ...
        }
    } else {
        error!("Bird has no wings! :(");
    }
}
```

### Casting

Like `Instance<T>`, an `Object<T>` maybe be cast into an `Object<U>` if `T` implements `CastObjectInto<U>`. You may implement this trait for your own kinds using the `safe_object_cast` macro:

```rust
// We expect every Bird to have a Creature component.
#[derive(Bundle)]
struct BirdBundle {
    bird: Bird,
    creature: Creature,
    name: Name,
    // ...
}

// Therefore, all birds may safely be assumed to be creatures:
safe_object_cast!(Bird => Creature);

// Birds can chirp.
fn chirp(bird: Object<Bird>) {
    // ...
}

// Creatures can find food.
fn find_food(creature: Object<Creature>) {
    // ...
}

// Birds chirp when they get hungry.
fn handle_hunger(bird: Object<Bird>) {
    chirp(bird);
    find_food(bird.cast_into()); // Safe!
}

```

Any `Object<T>` is safely convertible to `Object`.

You can define as many casts as you want. Any object kind may be cast into any other object kind as long as a `safe_object_cast` is defined for it.

### Filters

Like a standard [`Query`](https://bevyengine.org/docs/query/), you can filter `Objects` by passing a [`QueryFilter`](https://bevyengine.org/docs/query/#query-filter) to the `iter` method:

```rust
#[derive(Component)]
struct Flying;

fn update_flying_birds(birds: Objects<Bird, With<Flying>>) {
    for object in birds.iter() {
        // This system only iterates birds with `Flying` component.
    }
}
```