use std::{fmt, num::ParseIntError, str::FromStr};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    pub hours: u64,
    pub minutes: u8,
    pub seconds: u8,
}

impl Time {
    pub fn as_seconds(self) -> u64 {
        3600 * self.hours + 60 * self.minutes as u64 + self.seconds as u64
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.hours, self.minutes, self.seconds)
    }
}

impl FromStr for Time {
    type Err = ParseIntError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        let dot_pos = input.find('.');
        Ok(if let Some(dot) = dot_pos {
            let hours: u64 = input[0..dot].parse()?;
            let (minutes, seconds) = if dot + 1 < input.len() {
                let minutes_seconds = &input[(dot + 1)..];
                match minutes_seconds.len() {
                    0 => (0, 0),
                    1 | 2 => (minutes_seconds.parse()?, 0),
                    len => (
                        minutes_seconds[0..2].parse()?,
                        minutes_seconds[2..(len.min(4))].parse()?,
                    ),
                }
            } else {
                (0, 0)
            };
            Self {
                hours,
                minutes,
                seconds,
            }
        } else {
            let hours: u64 = input.parse()?;
            Self {
                hours,
                minutes: 0,
                seconds: 0,
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn time() {
        assert_eq!(
            Time::from_str("1"),
            Ok(Time {
                hours: 1,
                minutes: 0,
                seconds: 0
            })
        );
        assert_eq!(
            Time::from_str("137"),
            Ok(Time {
                hours: 137,
                minutes: 0,
                seconds: 0
            })
        );
        assert_eq!(
            Time::from_str("137."),
            Ok(Time {
                hours: 137,
                minutes: 0,
                seconds: 0
            })
        );
        assert_eq!(
            Time::from_str("137.2"),
            Ok(Time {
                hours: 137,
                minutes: 2,
                seconds: 0
            })
        );
        assert_eq!(
            Time::from_str("137.25"),
            Ok(Time {
                hours: 137,
                minutes: 25,
                seconds: 0
            })
        );
        assert_eq!(
            Time::from_str("137.259"),
            Ok(Time {
                hours: 137,
                minutes: 25,
                seconds: 9
            })
        );
        assert_eq!(
            Time::from_str("137.2597"),
            Ok(Time {
                hours: 137,
                minutes: 25,
                seconds: 97
            })
        );
        assert_eq!(
            Time::from_str("137.259723231"),
            Ok(Time {
                hours: 137,
                minutes: 25,
                seconds: 97
            })
        );
    }
}
