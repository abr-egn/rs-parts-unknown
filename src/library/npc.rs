use crate::{
    creature::{Creature},
    id_map::{Id, IdMap},
    npc::{self, NPC},
    part::{Part, PartTag},
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
 
        let mono = Monopod {
            kick_time: true,
            head, foot,
        };
        Creature::new_ids("Monopod", parts, Some(NPC {
            intent: mono.kick(),
            behavior: Box::new(mono),
        }))
    }

    fn kick(&self) -> npc::Intent {
        npc::Intent {
            name: "Kick".into(),
            from: Some(self.foot),
            cost: 1,
            kind: npc::IntentKind::Attack {
                damage: 10,
                range: npc::Range::Melee,
            },
        }
    }

    fn headbutt(&self) -> npc::Intent {
        npc::Intent {
            name: "Headbutt".into(),
            from: Some(self.head),
            cost: 1,
            kind: npc::IntentKind::Attack {
                damage: 5,
                range: npc::Range::Melee,
            },
        }
    }
}

impl npc::Behavior for Monopod {
    fn intent(&mut self, _world: &World, _id: Id<Creature>) -> Vec<npc::Intent> {
        self.kick_time = !self.kick_time;
        if self.kick_time {
            vec![self.kick(), self.headbutt()]
        } else {
            vec![self.headbutt(), self.kick()]
        }       
    }
}