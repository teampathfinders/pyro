use better_any::{Tid, tid};

use crate::{Component, request::Req, With, World, Without};

#[derive(Debug)]
struct Player {
    name: &'static str
}

tid!(Player);
impl<'t> Component<'t> for Player {}

#[derive(Debug)]
struct Alive; 

tid!(Alive);
impl<'t> Component<'t> for Alive {}

fn exclusive_system(query: Req<&mut Player, With<Alive>>) {
    // for player in &query {
    //     println!("{player:?}");
    // }
}

fn shared_system(req: Req<&Player, Without<Alive>>) {

}

fn empty_system(_: ()) {
    println!("I am an empty system");
}

#[test]
fn query_test() {
    let mut world = World::new();
    let id = world.spawn(Alive).id();   
    
    world.system(empty_system);
    world.run_all();
    // world.system(shared_system);
}