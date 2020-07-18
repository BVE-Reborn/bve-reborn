use crate::Time;
use smartstring::{LazyCompact, SmartString};
use std::str::FromStr;

macro_rules! flag_enum {
    ($name:ident, $ty:ty, $($variant:ident = $num:expr),*) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum $name {
            $($variant = $num,)*
        }
        impl FromStr for $name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.parse::<$ty>().map_err(|_| ())? {
                    $($num => Ok(Self::$variant),)*
                    _ => Err(()),
                }
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimetableSuffix {
    Day,
    Night,
}
impl FromStr for TimetableSuffix {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "day" => Ok(Self::Day),
            "night" => Ok(Self::Night),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrivalTimeState {
    Player(Option<Time>),
    AiStop,
    AllPass,
}
impl FromStr for ArrivalTimeState {
    type Err = !;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input: SmartString<LazyCompact> = input.trim().chars().flat_map(char::to_lowercase).collect();
        Ok(match input.chars().next() {
            Some('p' | 'l') => Self::AllPass,
            Some('b') => Self::AiStop,
            Some('s') => {
                if input.len() > 2 && input.chars().nth(1) == Some(':') {
                    Self::Player(input[2..].parse().ok())
                } else {
                    Self::Player(None)
                }
            }
            Some(_) => Self::Player(input.parse().ok()),
            None => Self::Player(None),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DepartureTimeState {
    Regular(Option<Time>),
    Terminal(Option<Time>),
    ChangeEnds(Option<Time>),
    Jump { index: u64, time: Option<Time> },
}
impl FromStr for DepartureTimeState {
    type Err = !;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input: SmartString<LazyCompact> = input.trim().chars().flat_map(char::to_lowercase).collect();
        Ok(match input.chars().next() {
            Some('=') => Self::Terminal(None),
            Some('t') => {
                if input.len() > 2 && input.chars().nth(1) == Some(':') {
                    Self::Terminal(input[2..].parse().ok())
                } else {
                    Self::Terminal(None)
                }
            }
            Some('c') => {
                if input.len() > 2 && input.chars().nth(1) == Some(':') {
                    Self::ChangeEnds(input[2..].parse().ok())
                } else {
                    Self::ChangeEnds(None)
                }
            }
            Some('j') => {
                if input.len() > 2 && input.chars().nth(1) == Some(':') {
                    let second_colon = input[2..].find(':').map(|v| v + 2);
                    let output: Option<Self> = try {
                        if let Some(second_colon) = second_colon {
                            if input.len() > second_colon + 1 {
                                Self::Jump {
                                    index: input[2..second_colon].parse().ok()?,
                                    time: input[(second_colon + 1)..].parse().ok(),
                                }
                            } else {
                                Self::Jump {
                                    index: input[2..second_colon].parse().ok()?,
                                    time: None,
                                }
                            }
                        } else {
                            None?
                        }
                    };
                    output.unwrap_or_else(|| Self::Regular(None))
                } else {
                    Self::Regular(None)
                }
            }
            Some(_) => Self::Regular(input.parse().ok()),
            None => Self::Regular(None),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StationDoorMode {
    Left = -1,
    None = 0,
    Right = 1,
    Both = 2,
}
impl FromStr for StationDoorMode {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input: SmartString<LazyCompact> = input.trim().chars().flat_map(char::to_lowercase).collect();
        match (input.chars().next(), input.parse::<i8>()) {
            (Some('l'), _) | (_, Ok(-1)) => Ok(Self::Left),
            (Some('n'), _) | (_, Ok(0)) => Ok(Self::None),
            (Some('r'), _) | (_, Ok(1)) => Ok(Self::Right),
            (Some('b'), _) => Ok(Self::Both),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemAtsMode {
    ATS = 0,
    ATC = 1,
}
impl FromStr for SystemAtsMode {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input: SmartString<LazyCompact> = input.trim().chars().flat_map(char::to_lowercase).collect();
        match (input.as_str(), input.parse::<u8>()) {
            ("ats", _) | (_, Ok(0)) => Ok(Self::ATS),
            ("atc", _) | (_, Ok(1)) => Ok(Self::ATC),
            _ => Err(()),
        }
    }
}
