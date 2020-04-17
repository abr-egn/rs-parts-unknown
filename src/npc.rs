use crate::{
    creature::{Creature},
    error::{Error, Result},
    event::Event,
    id_map::Id,
    world::World,
};

#[derive(Debug, Clone)]
pub struct Npc {
    next_motion: Option<Motion>,
    next_action: Option<Action>,
    behavior: Box<dyn Behavior>,
}

impl Npc {
    pub fn next_motion(&self) -> Option<&Motion> { self.next_motion.as_ref() }
    pub fn next_action(&self) -> Option<&Action> { self.next_action.as_ref() }

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

#[derive(Clone)]
pub struct Action {
    pub kind: ActionKind,
    pub run: fn(&mut World, Id<Creature>) -> Result<Vec<Event>>,
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("kind", &self.kind)
            .field("run", &(self.run as usize))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionKind {
    Attack,
}

trait Behavior: BehaviorClone + std::fmt::Debug + Send {
    fn next(&mut self, world: &World, id: Id<Creature>) -> (Option<Motion>, Option<Action>);
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

#[derive(Debug, Clone)]
pub struct Monopod {}

impl Behavior for Monopod {
    fn next(&mut self, _world: &World, _id: Id<Creature>) -> (Option<Motion>, Option<Action>) {
        let action = Action {
            kind: ActionKind::Attack,
            run: Monopod::kick,
        };
        (Some(Motion::ToMelee), Some(action))
    }
}

impl Monopod {
    fn kick(world: &mut World, id: Id<Creature>) -> Result<Vec<Event>> {
        check_run(world, id, "Fut", Range::Melee, 1)?;
        unimplemented!()
    }
}

fn check_run(world: &World, id: Id<Creature>, part: &str, range: Range, cost: i32) -> Result<()> {
    let creature = world.creatures().get(id).ok_or(Error::NoSuchCreature)?;
    if creature.cur_ap() < cost {
        return Err(Error::NotEnough);
    }
    if !creature.parts().values().any(|p| p.name == part && !p.dead) {
        return Err(Error::NoSuchPart);
    }
    let creature_pos = world.map().creatures().get(&id).ok_or(Error::OutOfBounds)?;
    let player_pos = world.map().creatures().get(&world.player_id()).ok_or(Error::OutOfBounds)?;
    let dist = creature_pos.distance_to(*player_pos);
    match range {
        Range::Melee => if dist != 1 { return Err(Error::Obstructed); }
    }

    Ok(())
}

enum Range {
    Melee,
    //Ranged { min: i32, max: i32 }
}