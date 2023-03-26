use std::{sync::Arc, marker::PhantomData, any::{TypeId, Any}, collections::HashMap};

use dashmap::DashMap;
use parking_lot::RwLock;

pub struct EntityId(usize);

pub struct Entity<'a> {
    world_state: Arc<WorldState>,
    id: EntityId,
    _marker: PhantomData<&'a ()>
}

pub trait Spawnable {
    fn insert_all(self, storage: &Components, entity: usize);
}

pub trait RefComponent {
    const MUTABLE: bool;    
}

impl<T: Component> RefComponent for &T {
    const MUTABLE: bool = false;
}

impl<T: Component> RefComponent for &mut T {
    const MUTABLE: bool = true;
}

pub trait Filters {

}

impl Filters for () {}
impl<T: Component> Filters for With<T> {}
impl<T: Component> Filters for Without<T> {}

pub struct With<T: Component> {
    _marker: PhantomData<T>
}

pub struct Without<T: Component> {
    _marker: PhantomData<T>
}

pub trait Requestable {
    const MUTABLE: bool;
}

impl<T: RefComponent> Requestable for T {
    const MUTABLE: bool = T::MUTABLE;
}

impl<T0: RefComponent, T1: RefComponent> Requestable for (T0, T1) {
    const MUTABLE: bool = T0::MUTABLE || T1::MUTABLE;
}

pub trait Component {

}

impl<T: Component + 'static> Spawnable for T {
    fn insert_all(self, storage: &Components, entity: usize) {
        storage.insert(self, entity);
    }
}
impl<T0: Component + 'static, T1: Component + 'static> Spawnable for (T0, T1) {
    fn insert_all(self, storage: &Components, entity: usize) {
        storage.insert(self.0, entity);
        storage.insert(self.1, entity);
    }
}

pub trait SystemParam {
    const MUTABLE: bool;

    type Item<'q>;

    fn fetch<'q>(state: &'q WorldState) -> Self::Item<'q>;
}   

impl<S: Requestable, F: Filters> SystemParam for Req<S, F> {
    const MUTABLE: bool = S::MUTABLE;

    type Item<'q> = Req<S, F>;

    fn fetch<'q>(state: &'q WorldState) -> Self::Item<'q> {
        todo!();
    }
}

pub trait SystemParams {
    const MUTABLE: bool;
}

impl<P: SystemParam> SystemParams for P {
    const MUTABLE: bool = P::MUTABLE;
}

impl<P0: SystemParam, P1: SystemParam> SystemParams for (P0, P1) {
    const MUTABLE: bool = P0::MUTABLE || P1::MUTABLE;
}

pub trait System {
    fn call(&self);
}

pub struct ParallelSystem<S, P: SystemParams> 
where
    ParallelSystem<S, P>: System
{
    f: S,
    _marker: PhantomData<P>
}

impl<S, P: SystemParams> ParallelSystem<S, P> 
where
    ParallelSystem<S, P>: System
{
    pub fn new(f: S) -> Self {
        Self {
            f, _marker: PhantomData
        }
    }
}

impl<S: Fn(P), P: SystemParam> System for ParallelSystem<S, P> {
    fn call(&self) {
        // (self.f)();
    }
}

impl<S: Fn(P0, P1), P0: SystemParam, P1: SystemParam> System for ParallelSystem<S, (P0, P1)> {
    fn call(&self) {

    }
}

pub struct Req<S, F = ()>
where
    S: Requestable,
    F: Filters,
{
    _marker: PhantomData<(S, F)>
}

impl<'q, S, F> IntoIterator for &'q Req<S, F> 
where
    S: Requestable,
    F: Filters
{
    type IntoIter = ReqIter<'q, S, F>;
    type Item = S;

    fn into_iter(self) -> Self::IntoIter {
        ReqIter { req: self }
    }
}

pub struct ReqIter<'q, S: Requestable, F: Filters> {
    req: &'q Req<S, F>
}

impl<'q, S: Requestable, F: Filters> Iterator for ReqIter<'q, S, F> {
    type Item = S;

    fn next(&mut self) -> Option<S> {
        todo!();
    }
}

pub trait IntoSystem<'a, S, Params> {
    fn into_boxed(self) -> Box<dyn System + 'a>;
}

impl<'a, S, P> IntoSystem<'a, S, P> for S 
where
    S: Fn(P) + 'a,
    P: SystemParam + 'a
{
    fn into_boxed(self) -> Box<dyn System + 'a> {
        if P::MUTABLE {
            todo!();
        } else {
            Box::new(ParallelSystem::new(self))
        }
    }
}

impl<'a, S, P0, P1> IntoSystem<'a, S, (P0, P1)> for S 
where
    S: Fn(P0, P1) + 'a,
    P0: SystemParam + 'a,
    P1: SystemParam + 'a
{
    fn into_boxed(self) -> Box<dyn System + 'a> {
        if <(P0, P1)>::MUTABLE {
            todo!();
        } else {
            Box::new(ParallelSystem::new(self))
        }
    }
}

#[derive(Default)]
pub struct Entities {
    mapping: RwLock<Vec<bool>>
}

impl Entities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&self) -> usize {
        let mut lock = self.mapping.write();
        lock
            .iter_mut()
            .enumerate()
            .find_map(|(i, v)| {
                if *v {
                    None
                } else {
                    *v = true;
                    Some(i)
                }
            })
            .or_else(|| {
                let len = lock.len();
                lock.push(true);
                Some(len)
            })
            .unwrap()
    }

    pub fn release(&self, entity: usize) {
        self.mapping.write()[entity] = false;
    }
}

pub trait Store {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct SpecializedStore<T: Component> {
    mapping: HashMap<usize, usize>,
    storage: Vec<Option<T>>
}

impl<T: Component> SpecializedStore<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, component: T, entity: usize) {
        let index = self.storage
            .iter_mut()
            .enumerate()
            .find_map(|(i, v)| if v.is_none() {
                Some(i)
            } else {
                None
            });

        if let Some(index) = index {
            self.mapping.insert(entity, index);
            self.storage[index] = Some(component);
        } else {
            let len = self.storage.len();
            self.mapping.insert(entity, len);
            self.storage.push(Some(component));
        }
    }
}

impl<T: Component> Default for SpecializedStore<T> {
    fn default() -> Self {
        Self {
            mapping: HashMap::new(),
            storage: Vec::new()
        }
    }
}

impl<T: Component + 'static> Store for SpecializedStore<T> {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct Components {
    storage: DashMap<TypeId, Box<dyn Store>>
}

impl Components {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Component + 'static>(&self, component: T, entity: usize) {
        let mut entry = self.storage
            .entry(TypeId::of::<T>())
            .or_insert_with(|| {
                Box::new(SpecializedStore::<T>::new())
            });

        let storage = entry.value_mut();
        let downcast = storage.as_any_mut().downcast_mut::<SpecializedStore<T>>().unwrap();
        downcast.insert(component, entity);
    }
}

#[derive(Default)]
pub struct WorldState {
    entities: Entities,
    components: Components
}

#[derive(Default)]
pub struct World {
    state: Arc<WorldState>
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn spawn(&self, spawnable: impl Spawnable) -> Entity {
        let entity_id = self.state.entities.acquire();
        spawnable.insert_all(&self.state.components, entity_id);

        Entity {
            id: EntityId(entity_id),
            world_state: self.state.clone(),
            _marker: PhantomData
        }
    }

    pub fn system<'a, S, P: SystemParams>(&'a self, systems: impl IntoSystem<'a, S, P>) {
        
    }

    pub fn run_all(&self) {

    }
}