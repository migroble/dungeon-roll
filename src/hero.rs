#[derive(Debug)]
pub enum HeroLevel {
    Novice,
    Expert,
}

#[derive(Debug)]
pub enum HeroType {
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
    hero: HeroType,
    level: HeroLevel,
    xp: u64,
    ult_used: bool,
}

impl Hero {
    pub fn new(hero: HeroType) -> Self {
        Self {
            hero,
            level: HeroLevel::Novice,
            xp: 0,
            ult_used: false,
        }
    }
}
