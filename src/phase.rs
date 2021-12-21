#[derive(Debug, PartialEq, Eq)]
pub enum MonsterPhase {
    SelectAlly,
    SelectReroll,
    ConfirmReroll,
    SelectMonster,
    ConfirmCombat,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LootPhase {
    SelectAlly,
    SelectLoot,
    ConfirmLoot,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Phase {
    Monster(MonsterPhase),
    Loot(LootPhase),
    Dragon,
    Regroup,
}
