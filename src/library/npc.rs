use crate::{
    creature::{Creature, Part, PartTag},
    id_map::{Id, IdMap},
    npc,
    world::World,
};

#[derive(Debug, Clone)]
pub struct Monopod {
    kick_time: bool,
    head: Id<Part>,
    foot: Id<Part>,
}

impl Monopod {
    pub fn creature() -> Creature {
        let mut parts = IdMap::new();
        let head = parts.add(Part {
            thought: 1,
            ..Part::new(
                "Hed",
                &[PartTag::Head, PartTag::Flesh, PartTag::Vital],
                20)
        });
        let foot = parts.add(Part {
            mp: 3,
            ..Part::new(
                "Fut", 
                &[PartTag::Limb, PartTag::Flesh, PartTag::Leg, PartTag::Open],
                20)
        });
        
        Creature::new_npc("Monopod", parts, Monopod {
            kick_time: true,
            head, foot,
        })
    }
}

impl npc::Behavior for Monopod {
    fn next(&mut self, _world: &World, _id: Id<Creature>) -> (Option<npc::Motion>, Option<npc::Intent>) {
        let intent = if self.kick_time {
            Some(npc::Intent {
                from: self.foot,
                cost: 1,
                kind: npc::IntentKind::Attack { base_damage: 10, range: npc::Range::Melee },
            })
        } else { None };
        self.kick_time = !self.kick_time;
        (Some(npc::Motion::ToMelee), intent)
    }
}