use crate::{
    card::Card,
    creature::{Creature, Part},
    id_map::Id,
    world::World,
};

#[derive(Debug, Clone)]
pub struct Npc {
    next_motion: Motion,
    next_action: Action,
    behavior: Box<dyn Behavior>,
}

impl Npc {
    pub fn next_motion(&self) -> &Motion { &self.next_motion }
    pub fn next_action(&self) -> &Action { &self.next_action }

    pub fn update(&mut self, world: &World, id: Id<Creature>) {
        let (motion, action) = self.behavior.next(world, id);
        self.next_motion = motion;
        self.next_action = action;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motion {
    ToMelee,
    // TODO: ToRanged { min: i32, max: i32 },
    // TODO: ToCover,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    kind: ActionKind,
    part: Id<Part>,
    card: Id<Card>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionKind {
    Attack,
}

trait Behavior: BehaviorClone + std::fmt::Debug + Send {
    fn next(&mut self, world: &World, id: Id<Creature>) -> (Motion, Action);
}

trait BehaviorClone {
    fn clone_box(&self) -> Box<dyn Behavior>;
}

impl<T> BehaviorClone for T
where T: 'static + Behavior + Clone,
{
    fn clone_box(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Behavior> {
    fn clone(&self) -> Self { self.clone_box() }
}