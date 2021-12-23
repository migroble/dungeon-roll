#[derive(Debug, PartialEq, Eq)]
pub enum Phase {
    Setup,
    Monster(MonsterPhase),
    Loot(LootPhase),
    Dragon(DragonPhase),
    Regroup,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MonsterPhase {
    SelectAlly,
    SelectReroll(Reroll),
    ConfirmReroll,
    SelectMonster,
    ConfirmCombat,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reroll {
    Ally,
    Monster,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LootPhase {
    SelectAlly,
    SelectLoot,
    ConfirmLoot,
    SelectGraveyard,
    ConfirmGraveyard,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DragonPhase {
    SelectAlly,
    Confirm,
}
