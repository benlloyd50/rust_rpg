use std::ops::DerefMut;

use specs::{storage::MaskedStorage, Component, Entity, Storage};

pub trait MaybeInsert<T: Component> {
    /// Inserts a component wrapped in an Option if it is Some(T)
    fn maybe_insert(&mut self, _onto: Entity, _component: Option<T>) {}
}

impl<'e, T, D> MaybeInsert<T> for Storage<'e, T, D>
where
    T: Component,
    D: DerefMut<Target = MaskedStorage<T>>,
{
    fn maybe_insert(&mut self, onto: Entity, maybe_component: Option<T>) {
        if let Some(component) = maybe_component {
            let _ = self.insert(onto, component);
        }
    }
}
