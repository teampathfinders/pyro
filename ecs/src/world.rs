use std::sync::Arc;

use parking_lot::RwLock;

use crate::{entity::{Entities, Entity, EntityId}, component::{Components, Component, Spawnable}, system::{Systems, SystemParams, IntoSystem}};

#[derive(Default)]
pub struct WorldState {
    pub(crate) entities: Entities,
    pub(crate) components: Components
}

impl WorldState {
    #[inline]
    pub fn entity_has<T: Component + 'static>(&self, entity: usize) -> bool {
        self.components.entity_has::<T>(entity)
    }
}

#[derive(Default)]
pub struct World {
    state: Arc<RwLock<WorldState>>,
    systems: Systems
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn spawn(&self, spawnable: impl Spawnable) -> Entity {
        let entity_id = self.state.read().entities.acquire();
        spawnable.insert_all(&mut self.state.write().components, entity_id);

        Entity {
            id: EntityId(entity_id),
            world_state: self.state.clone()
        }
    }

    pub fn system<S, P: SystemParams>(&self, system: impl IntoSystem<S, P>) {
        self.systems.insert(system);
    }

    pub async fn run_all(&self) {
        self.systems.run_all(&self.state).await;
    }
}