use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Treasure {
    VorpalSword,
    Talisman,
    ScepterOfPower,
    ThievesTools,
    Scroll,
    RingOfInvisiblity,
    DragonScales,
    Potion,
    DragonBait,
    TownPortal,
}

lazy_static! {
    pub static ref TREASURE: Vec<Treasure> = {
        let mut amounts = HashMap::new();
        amounts.insert(Treasure::VorpalSword, 3);
        amounts.insert(Treasure::Talisman, 3);
        amounts.insert(Treasure::ScepterOfPower, 3);
        amounts.insert(Treasure::ThievesTools, 3);
        amounts.insert(Treasure::Scroll, 3);
        amounts.insert(Treasure::RingOfInvisiblity, 4);
        amounts.insert(Treasure::DragonScales, 6);
        amounts.insert(Treasure::Potion, 3);
        amounts.insert(Treasure::DragonBait, 4);
        amounts.insert(Treasure::TownPortal, 4);

        let mut treasure = Vec::with_capacity(36);
        amounts.iter().for_each(|(item, amt)| {
            (0..*amt).for_each(|_| {
                treasure.push(item.clone());
            })
        });

        treasure
    };
}
