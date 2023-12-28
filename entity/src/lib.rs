mod component;
mod entity;
mod query;
mod system;
mod world;

use crate::component::Component;
use crate::query::{Query, QueryBundle, With};
use crate::world::World;
use std::fmt::Debug;

#[derive(Debug)]
struct Health {
    pub value: f32,
}

impl Component for Health {}

#[derive(Debug)]
struct Alive;

impl Component for Alive {}

#[derive(Debug)]
struct UniqueId(usize);

impl Component for UniqueId {}

fn system1(query: Query<(&UniqueId, &mut Health), With<Alive>>) {
    for (id, health) in query {
        dbg!(id);
        dbg!(health);
    }
}

fn system2(query: Query<(&UniqueId, &Health), With<Alive>>, query2: Query<&Alive>) {

}

#[test]
fn test1() {
    let mut world = World::new();
    world.system(system1);
    // world.system(system2);
    world.spawn((Alive, Health { value: 1.0 }));

    // dbg!(world);
}
