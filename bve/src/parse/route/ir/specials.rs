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
                            Self::Jump {
                                index: input[2..].parse().ok()?,
                                time: None,
                            }
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

#[derive(Debug, Clone, PartialEq)]
pub enum FormRailIndex2Data {
    Current(u64),
    Left,
    Right,
}
impl FromStr for FormRailIndex2Data {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input: SmartString<LazyCompact> = input.trim().chars().flat_map(char::to_lowercase).collect();
        Ok(match input.chars().next() {
            Some('l') => Self::Left,
            Some('r') => Self::Right,
            _ => Self::Current(input.parse().map_err(|_| ())?),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextMarkerColor {
    Black,
    Gray,
    White,
    Red,
    Orange,
    Green,
    Blue,
    Magenta,
}
impl FromStr for TextMarkerColor {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input: SmartString<LazyCompact> = input.trim().chars().flat_map(char::to_lowercase).collect();
        match input.as_str() {
            "black" => Ok(Self::Black),
            "gray" => Ok(Self::Gray),
            "white" => Ok(Self::White),
            "red" => Ok(Self::Red),
            "orange" => Ok(Self::Orange),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            "magenta" => Ok(Self::Magenta),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn timetable_suffix() {
        assert_eq!(TimetableSuffix::from_str("day"), Ok(TimetableSuffix::Day));
        assert_eq!(TimetableSuffix::from_str("night"), Ok(TimetableSuffix::Night));

        assert_eq!(TimetableSuffix::from_str("22"), Err(()));
        assert_eq!(TimetableSuffix::from_str("ATQ"), Err(()));
        assert_eq!(TimetableSuffix::from_str("  "), Err(()));
    }

    #[test]
    fn arrival_time_state() {
        assert_eq!(
            ArrivalTimeState::from_str("10.2346"),
            Ok(ArrivalTimeState::Player(Some(Time {
                hours: 10,
                minutes: 23,
                seconds: 46,
            })))
        );
        assert_eq!(ArrivalTimeState::from_str(" "), Ok(ArrivalTimeState::Player(None)));
        assert_eq!(ArrivalTimeState::from_str("P"), Ok(ArrivalTimeState::AllPass));
        assert_eq!(ArrivalTimeState::from_str("L"), Ok(ArrivalTimeState::AllPass));
        assert_eq!(ArrivalTimeState::from_str("B"), Ok(ArrivalTimeState::AiStop));
        assert_eq!(ArrivalTimeState::from_str("S"), Ok(ArrivalTimeState::Player(None)));
        assert_eq!(
            ArrivalTimeState::from_str("S:10.2346"),
            Ok(ArrivalTimeState::Player(Some(Time {
                hours: 10,
                minutes: 23,
                seconds: 46,
            })))
        );

        assert_eq!(ArrivalTimeState::from_str("1t45"), Ok(ArrivalTimeState::Player(None)));
        assert_eq!(ArrivalTimeState::from_str("ATQ"), Ok(ArrivalTimeState::Player(None)));
        assert_eq!(ArrivalTimeState::from_str(" "), Ok(ArrivalTimeState::Player(None)));
    }

    #[test]
    fn departure_time_state() {
        assert_eq!(
            DepartureTimeState::from_str("10.2346"),
            Ok(DepartureTimeState::Regular(Some(Time {
                hours: 10,
                minutes: 23,
                seconds: 46,
            })))
        );
        assert_eq!(DepartureTimeState::from_str(" "), Ok(DepartureTimeState::Regular(None)));
        assert_eq!(
            DepartureTimeState::from_str("T"),
            Ok(DepartureTimeState::Terminal(None))
        );
        assert_eq!(
            DepartureTimeState::from_str("="),
            Ok(DepartureTimeState::Terminal(None))
        );
        assert_eq!(
            DepartureTimeState::from_str("T:10.2346"),
            Ok(DepartureTimeState::Terminal(Some(Time {
                hours: 10,
                minutes: 23,
                seconds: 46,
            })))
        );
        assert_eq!(
            DepartureTimeState::from_str("C"),
            Ok(DepartureTimeState::ChangeEnds(None))
        );
        assert_eq!(
            DepartureTimeState::from_str("C:10.2346"),
            Ok(DepartureTimeState::ChangeEnds(Some(Time {
                hours: 10,
                minutes: 23,
                seconds: 46,
            })))
        );
        assert_eq!(
            DepartureTimeState::from_str("J:72"),
            Ok(DepartureTimeState::Jump { index: 72, time: None })
        );
        assert_eq!(
            DepartureTimeState::from_str("J:72:10.2346"),
            Ok(DepartureTimeState::Jump {
                index: 72,
                time: Some(Time {
                    hours: 10,
                    minutes: 23,
                    seconds: 46,
                }),
            })
        );

        assert_eq!(
            DepartureTimeState::from_str("T2"),
            Ok(DepartureTimeState::Terminal(None))
        );
        assert_eq!(
            DepartureTimeState::from_str("C2"),
            Ok(DepartureTimeState::ChangeEnds(None))
        );
        assert_eq!(
            DepartureTimeState::from_str("J2"),
            Ok(DepartureTimeState::Regular(None))
        );
        assert_eq!(
            DepartureTimeState::from_str("J:-22"),
            Ok(DepartureTimeState::Regular(None))
        );
        assert_eq!(
            DepartureTimeState::from_str("J:22:H"),
            Ok(DepartureTimeState::Jump { index: 22, time: None })
        );
        assert_eq!(DepartureTimeState::from_str(" "), Ok(DepartureTimeState::Regular(None)));
    }

    #[test]
    fn station_door_mode() {
        assert_eq!(StationDoorMode::from_str("L"), Ok(StationDoorMode::Left));
        assert_eq!(StationDoorMode::from_str("-1"), Ok(StationDoorMode::Left));
        assert_eq!(StationDoorMode::from_str("N"), Ok(StationDoorMode::None));
        assert_eq!(StationDoorMode::from_str("0"), Ok(StationDoorMode::None));
        assert_eq!(StationDoorMode::from_str("R"), Ok(StationDoorMode::Right));
        assert_eq!(StationDoorMode::from_str("1"), Ok(StationDoorMode::Right));
        assert_eq!(StationDoorMode::from_str("B"), Ok(StationDoorMode::Both));

        assert_eq!(StationDoorMode::from_str("-23"), Err(()));
        assert_eq!(StationDoorMode::from_str("TWE"), Err(()));
        assert_eq!(StationDoorMode::from_str("  "), Err(()));
    }

    #[test]
    fn system_ats_mode() {
        assert_eq!(SystemAtsMode::from_str("ATS"), Ok(SystemAtsMode::ATS));
        assert_eq!(SystemAtsMode::from_str("0"), Ok(SystemAtsMode::ATS));
        assert_eq!(SystemAtsMode::from_str("ATC"), Ok(SystemAtsMode::ATC));
        assert_eq!(SystemAtsMode::from_str("1"), Ok(SystemAtsMode::ATC));

        assert_eq!(SystemAtsMode::from_str("-23"), Err(()));
        assert_eq!(SystemAtsMode::from_str("ATQ"), Err(()));
        assert_eq!(SystemAtsMode::from_str("  "), Err(()));
    }

    #[test]
    fn form_rail_index2_data() {
        assert_eq!(FormRailIndex2Data::from_str("2"), Ok(FormRailIndex2Data::Current(2)));
        assert_eq!(FormRailIndex2Data::from_str("15"), Ok(FormRailIndex2Data::Current(15)));
        assert_eq!(FormRailIndex2Data::from_str("L"), Ok(FormRailIndex2Data::Left));
        assert_eq!(FormRailIndex2Data::from_str("R"), Ok(FormRailIndex2Data::Right));

        assert_eq!(FormRailIndex2Data::from_str("-23"), Err(()));
        assert_eq!(FormRailIndex2Data::from_str("oops"), Err(()));
        assert_eq!(FormRailIndex2Data::from_str(" "), Err(()));
    }

    #[test]
    fn text_marker_color() {
        assert_eq!(TextMarkerColor::from_str("black"), Ok(TextMarkerColor::Black));
        assert_eq!(TextMarkerColor::from_str("gray"), Ok(TextMarkerColor::Gray));
        assert_eq!(TextMarkerColor::from_str("white"), Ok(TextMarkerColor::White));
        assert_eq!(TextMarkerColor::from_str("red"), Ok(TextMarkerColor::Red));
        assert_eq!(TextMarkerColor::from_str("orange"), Ok(TextMarkerColor::Orange));
        assert_eq!(TextMarkerColor::from_str("green"), Ok(TextMarkerColor::Green));
        assert_eq!(TextMarkerColor::from_str("blue"), Ok(TextMarkerColor::Blue));
        assert_eq!(TextMarkerColor::from_str("magenta"), Ok(TextMarkerColor::Magenta));

        assert_eq!(TextMarkerColor::from_str("grue"), Err(()));
        assert_eq!(TextMarkerColor::from_str(""), Err(()));
        assert_eq!(TextMarkerColor::from_str("asfa"), Err(()));
    }
}
