use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
};

lazy_static! {
    static ref FLAVOR_STYLE: Style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::ITALIC | Modifier::DIM);
}

pub trait Dice {
    fn nth(n: u64) -> Self;
    fn faces() -> u64;
}

pub trait Render {
    fn style(&self) -> Style;

    fn symbol(&self) -> &'static str;

    fn render(&self) -> Text {
        Text::styled(self.symbol(), self.style())
    }

    fn combat_info(&self) -> Spans;

    fn loot_info(&self) -> Spans;

    fn flavor_text(&self) -> Spans;
}

#[derive(Debug, Dice, PartialEq)]
pub enum Ally {
    Fighter,
    Cleric,
    Mage,
    Champion,
    Thief,
    Scroll,
}

impl Render for Ally {
    fn style(&self) -> Style {
        match self {
            Ally::Fighter => Style::default().fg(Color::Green),
            Ally::Cleric => Style::default().fg(Color::DarkGray),
            Ally::Mage => Style::default().fg(Color::Blue),
            Ally::Thief => Style::default().fg(Color::Rgb(128, 0, 128)),
            Ally::Champion => Style::default().fg(Color::Yellow),
            Ally::Scroll => Style::default().fg(Color::Rgb(255, 165, 0)),
        }
    }

    fn symbol(&self) -> &'static str {
        match self {
            Ally::Fighter => "F",
            Ally::Cleric | Ally::Champion => "C",
            Ally::Mage => "M",
            Ally::Thief => "T",
            Ally::Scroll => "S",
        }
    }

    fn combat_info(&self) -> Spans {
        match self {
            Ally::Fighter => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Fighter", self.style()),
                Span::raw(" defeats one "),
                Span::styled("Skeleton", Monster::Skeleton.style()),
                Span::raw(", one "),
                Span::styled("Ooze", Monster::Ooze.style()),
                Span::raw(", or any number of "),
                Span::styled("Goblins", Monster::Goblin.style()),
            ]),
            Ally::Cleric => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Cleric", self.style()),
                Span::raw(" defeats one "),
                Span::styled("Goblin", Monster::Goblin.style()),
                Span::raw(", one "),
                Span::styled("Ooze", Monster::Ooze.style()),
                Span::raw(", or any number of "),
                Span::styled("Skeletons", Monster::Skeleton.style()),
            ]),
            Ally::Mage => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Mage", self.style()),
                Span::raw(" defeats one "),
                Span::styled("Goblin", Monster::Goblin.style()),
                Span::raw(", one "),
                Span::styled("Skeleton", Monster::Skeleton.style()),
                Span::raw(", or any number of "),
                Span::styled("Oozes", Monster::Ooze.style()),
            ]),
            Ally::Thief => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Thief", self.style()),
                Span::raw(" defeats one "),
                Span::styled("Goblin", Monster::Goblin.style()),
                Span::raw(", one "),
                Span::styled("Skeleton", Monster::Skeleton.style()),
                Span::raw(", or one "),
                Span::styled("Oozes", Monster::Ooze.style()),
            ]),
            Ally::Champion => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Champion", self.style()),
                Span::raw(" may be used to defeat any number of "),
                Span::styled("Goblins", Monster::Goblin.style()),
                Span::raw(", any number of "),
                Span::styled("Skeletons", Monster::Skeleton.style()),
                Span::raw(", or any number of "),
                Span::styled("Oozes", Monster::Ooze.style()),
            ]),
            Ally::Scroll => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Scroll", self.style()),
                Span::raw(" may be used to re-roll any number of Dungeon and Party dice except for Dragon faces"),
            ]),
        }
    }

    fn loot_info(&self) -> Spans {
        match self {
            Ally::Fighter => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Fighter", self.style()),
                Span::raw(" may be used to open one "),
                Span::styled("Chests", Monster::Chest.style()),
                Span::raw(" or quaff any number of "),
                Span::styled("Potions", Monster::Potion.style()),
            ]),
            Ally::Cleric => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Cleric", self.style()),
                Span::raw(" may be used to open one "),
                Span::styled("Chests", Monster::Chest.style()),
                Span::raw(" or quaff any number of "),
                Span::styled("Potions", Monster::Potion.style()),
            ]),
            Ally::Mage => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Mage", self.style()),
                Span::raw(" may be used to open one "),
                Span::styled("Chests", Monster::Chest.style()),
                Span::raw(" or quaff any number of "),
                Span::styled("Potions", Monster::Potion.style()),
            ]),
            Ally::Thief => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Thief", self.style()),
                Span::raw(" may be used to open any number of "),
                Span::styled("Chests", Monster::Chest.style()),
                Span::raw(" or quaff any number of "),
                Span::styled("Potions", Monster::Potion.style()),
            ]),
            Ally::Champion => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Champion", self.style()),
                Span::raw(" may be used to open one "),
                Span::styled("Chests", Monster::Chest.style()),
                Span::raw(" or quaff any number of "),
                Span::styled("Potions", Monster::Potion.style()),
            ]),
            Ally::Scroll => Spans::from(vec![
                Span::raw("The "),
                Span::styled("Scroll", self.style()),
                Span::raw(" may be used to quaff any number of "),
                Span::styled("Potions", Monster::Potion.style()),
            ]),
        }
    }

    fn flavor_text(&self) -> Spans {
        match self {
            Ally::Fighter => Spans::from(Span::styled(
                "Who send all these babies to fight?",
                *FLAVOR_STYLE,
            )),
            Ally::Cleric => Spans::from(Span::styled("I kick ass for the Lord!", *FLAVOR_STYLE)),
            Ally::Mage => Spans::from(Span::styled(
                "Master of the arcane, relegated to disposing of goo",
                *FLAVOR_STYLE,
            )),
            Ally::Thief => Spans::from(Span::styled(
                "Thief is not the one who steals, but the one that is caught",
                *FLAVOR_STYLE,
            )),
            Ally::Champion => Spans::from(Span::styled(
                "Monster! You have no bearing, and no grace or courage!",
                *FLAVOR_STYLE,
            )),
            Ally::Scroll => Spans::from(Span::styled(
                "Never trust anyone who has not brought a scroll with them",
                *FLAVOR_STYLE,
            )),
        }
    }
}

impl Ally {
    pub fn is_companion(&self) -> bool {
        !matches!(self, Ally::Scroll)
    }
}

#[derive(Debug, Dice, PartialEq, Clone)]
pub enum Monster {
    Goblin,
    Skeleton,
    Ooze,
    Dragon,
    Chest,
    Potion,
}

impl Render for Monster {
    fn style(&self) -> Style {
        match self {
            Monster::Goblin => Style::default().fg(Color::Green),
            Monster::Skeleton => Style::default().fg(Color::DarkGray),
            Monster::Ooze => Style::default().fg(Color::Blue),
            Monster::Dragon => Style::default().fg(Color::Red),
            Monster::Chest => Style::default().fg(Color::Rgb(128, 0, 128)),
            Monster::Potion => Style::default().fg(Color::Rgb(255, 165, 0)),
        }
    }

    fn symbol(&self) -> &'static str {
        match self {
            Monster::Goblin => "G",
            Monster::Skeleton => "S",
            Monster::Ooze => "O",
            Monster::Dragon => "D",
            Monster::Chest => "C",
            Monster::Potion => "P",
        }
    }

    fn combat_info(&self) -> Spans {
        self.info()
    }

    fn loot_info(&self) -> Spans {
        self.info()
    }

    fn flavor_text(&self) -> Spans {
        match self {
            Monster::Goblin => Spans::from(Span::styled(
                "Don't let them gang up on you!",
                *FLAVOR_STYLE,
            )),
            Monster::Skeleton => Spans::from(Span::styled(
                "They appear very strong, they must have drunk a lot of milk",
                *FLAVOR_STYLE,
            )),
            Monster::Ooze => Spans::from(Span::styled(
                "Gooey and sticky, very hard to get off your clothes",
                *FLAVOR_STYLE,
            )),
            Monster::Dragon => Spans::from("If you can read this, you found a bug"),
            Monster::Chest => Spans::from(Span::styled("Don't be a mimic, please", *FLAVOR_STYLE)),
            Monster::Potion => Spans::from(Span::styled("This can't taste good", *FLAVOR_STYLE)),
        }
    }
}

impl Monster {
    pub fn is_monster(&self) -> bool {
        matches!(self, Monster::Goblin | Monster::Skeleton | Monster::Ooze)
    }

    pub fn is_loot(&self) -> bool {
        matches!(self, Monster::Chest | Monster::Potion)
    }

    fn info(&self) -> Spans {
        match self {
            Monster::Goblin => Spans::from(vec![
                Span::styled("Goblins", self.style()),
                Span::raw(" are small humanoids that dwell in shallow underground lairs"),
            ]),
            Monster::Skeleton => Spans::from(vec![
                Span::styled("Skeletons", self.style()),
                Span::raw(" are undead creatures reanimated by necromantic magic"),
            ]),
            Monster::Ooze => Spans::from(vec![
                Span::styled("Oozes", self.style()),
                Span::raw(
                    " are creatures that resemble amorphous blobs and dwell in the underground",
                ),
            ]),
            Monster::Dragon => Spans::from("If you can read this, you found a bug"),
            Monster::Chest => Spans::from(vec![
                Span::styled("Chests", self.style()),
                Span::raw(" contain treasures that may aid you in your quest"),
            ]),
            Monster::Potion => Spans::from(vec![
                Span::styled("Potions", self.style()),
                Span::raw(" may be used to bring an ally back from the graveyard"),
            ]),
        }
    }
}
