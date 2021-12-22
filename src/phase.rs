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
    SelectGraveyard,
    ConfirmGraveyard,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Phase {
    Setup,
    Monster(MonsterPhase),
    Loot(LootPhase),
    Dragon,
    Regroup,
}
