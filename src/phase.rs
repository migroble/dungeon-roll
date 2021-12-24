#[derive(Debug, PartialEq, Eq)]
pub enum Phase {
    Setup,
    Monster(Monster),
    Loot(Loot),
    Dragon(Dragon),
    EmptyDungeon,
    Regroup(Regroup),
    Victory,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, PartialEq, Eq)]
pub enum Monster {
    SelectAlly,
    SelectReroll(Reroll),
    ConfirmReroll,
    SelectMonster,
    ConfirmCombat,
    Defeat,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reroll {
    Ally,
    Monster,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, PartialEq, Eq)]
pub enum Loot {
    SelectAlly,
    SelectLoot,
    ConfirmLoot,
    SelectGraveyard,
    ConfirmGraveyard,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Dragon {
    SelectAlly,
    Confirm,
    Defeat,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Regroup {
    Continue,
    ContinueSetup,
    End,
    EndSetup,
}
