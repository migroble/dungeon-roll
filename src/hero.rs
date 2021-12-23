#[derive(Debug)]
pub enum Level {
    Novice,
    Expert,
}

#[derive(Debug)]
pub enum Type {
    Bard,
    Battlemage,
    Beguiler,
    Chieftain,
    Commander,
    DragonSlayer,
    Necromancer,
    Paladin,
}

#[derive(Debug)]
pub struct Hero {
    hero: Type,
    level: Level,
    xp: u64,
    ult_used: bool,
}

impl Hero {
    pub fn new(hero: Type) -> Self {
        Self {
            hero,
            level: Level::Novice,
            xp: 0,
            ult_used: false,
        }
    }
}
