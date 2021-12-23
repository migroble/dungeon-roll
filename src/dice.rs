use tui::{
    style::{Color, Style},
    text::Text,
};

pub trait Dice {
    fn nth(n: u64) -> Self;
    fn faces() -> u64;
}

pub trait Render {
    fn render(&self) -> Text;
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
    fn render(&self) -> Text {
        match self {
            Ally::Fighter => Text::styled("F", Style::default().fg(Color::Green)),
            Ally::Cleric => Text::styled("C", Style::default().fg(Color::Gray)),
            Ally::Mage => Text::styled("M", Style::default().fg(Color::Blue)),
            Ally::Champion => Text::styled("C", Style::default().fg(Color::Yellow)),
            Ally::Thief => Text::styled("T", Style::default().fg(Color::Rgb(128, 0, 128))),
            Ally::Scroll => Text::styled("S", Style::default().fg(Color::Rgb(255, 165, 0))),
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
    fn render(&self) -> Text {
        match self {
            Monster::Goblin => Text::styled("G", Style::default().fg(Color::Green)),
            Monster::Skeleton => Text::styled("S", Style::default().fg(Color::Gray)),
            Monster::Ooze => Text::styled("O", Style::default().fg(Color::Blue)),
            Monster::Dragon => Text::styled("D", Style::default().fg(Color::Red)),
            Monster::Chest => Text::styled("C", Style::default().fg(Color::Rgb(128, 0, 128))),
            Monster::Potion => Text::styled("P", Style::default().fg(Color::Rgb(255, 165, 0))),
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
}
