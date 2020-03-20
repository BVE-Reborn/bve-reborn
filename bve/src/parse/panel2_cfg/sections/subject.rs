use crate::parse::kvp::FromKVPValue;
use crate::parse::{util, PrettyPrintResult};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::combinator::{map_res, opt};
use nom::sequence::preceded;
use nom::IResult;
use std::io;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Subject {
    pub target: SubjectTarget,
    pub digit: Option<u64>,
}

impl Subject {
    #[must_use]
    pub const fn from_target(target: SubjectTarget) -> Self {
        Self { target, digit: None }
    }
}

impl FromKVPValue for Subject {
    fn from_kvp_value(value: &str) -> Option<Self> {
        match parse_subject(value) {
            Ok(("", o)) => Some(o),
            _ => None,
        }
    }
}

impl PrettyPrintResult for Subject {
    fn fmt(&self, indent: usize, out: &mut dyn io::Write) -> io::Result<()> {
        writeln!(
            out,
            "Target: {}",
            match self.target {
                SubjectTarget::Acceleration => "Acceleration",
                SubjectTarget::Ats(i) => return write!(out, "Ats #{}", i),
                SubjectTarget::LocoBrakeCylinder => "Loco Brake Cylinder",
                SubjectTarget::BrakeCylinder => "Brake Cylinder",
                SubjectTarget::LocoBrakePipe => "Loco Brake Pipe",
                SubjectTarget::BrakePipe => "Brake Pipe",
                SubjectTarget::Brake => "Brake",
                SubjectTarget::LocoBrake => "Loco Brake",
                SubjectTarget::ConstSpeedSystem => "Const Speed System",
                SubjectTarget::Door => "Door",
                SubjectTarget::DoorLeft(i) => return write!(out, "Left Door #{}", i),
                SubjectTarget::DoorRight(i) => return write!(out, "Right Door #{}", i),
                SubjectTarget::DoorButtonLeft => "Left Door Button",
                SubjectTarget::DoorButtonRight => "Right Door Button",
                SubjectTarget::EqualizingReservoir => "Equalizing Reservoir",
                SubjectTarget::Hour => "Hours",
                SubjectTarget::KilometersPerHour => "Speed (kph)",
                SubjectTarget::Minute => "Minutes",
                SubjectTarget::MotorAcceleration => "Motor Acceleration",
                SubjectTarget::MilesPerHour => "Speed (mph)",
                SubjectTarget::MainReservoir => "Main Reservoir",
                SubjectTarget::MetersPerSecond => "Speed (m/s)",
                SubjectTarget::PowerNotch => "Power Notch",
                SubjectTarget::Reverser => "Reverser",
                SubjectTarget::StraightAirPipe => "Straight Air Pipe",
                SubjectTarget::Second => "Seconds",
                SubjectTarget::True => "True",
                SubjectTarget::Klaxon => "Klaxon",
                SubjectTarget::PrimaryKlaxon => "Primary Klaxon",
                SubjectTarget::SecondaryKlaxon => "SecondaryKlaxon",
                SubjectTarget::MusicKlaxon => "Music Klaxon",
                SubjectTarget::PassAlarm => "Pass Alarm",
                SubjectTarget::PilotLamp => "Pilot Lamp",
                SubjectTarget::StationAdjustAlarm => "StationAdjustAlarm",
            },
        )?;
        if let Some(d) = self.digit {
            util::indent(indent + 1, out)?;
            write!(out, "Digit: {}", d)?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubjectTarget {
    Acceleration,
    Ats(u64),
    LocoBrakeCylinder,
    BrakeCylinder,
    LocoBrakePipe,
    BrakePipe,
    Brake,
    LocoBrake,
    ConstSpeedSystem,
    Door,
    DoorLeft(u64),
    DoorRight(u64),
    DoorButtonLeft,
    DoorButtonRight,
    EqualizingReservoir,
    Hour,
    KilometersPerHour,
    Minute,
    MotorAcceleration,
    MilesPerHour,
    MainReservoir,
    MetersPerSecond,
    PowerNotch,
    Reverser,
    StraightAirPipe,
    Second,
    True,
    Klaxon,
    PrimaryKlaxon,
    SecondaryKlaxon,
    MusicKlaxon,
    PassAlarm,
    PilotLamp,
    StationAdjustAlarm,
}

macro_rules! variant {
    ($life:lifetime, $name:literal, $variant:ident) => {
        move |s: &$life str| -> IResult<&$life str, SubjectTarget> {
            tag($name)(s).map(|(i, _o)| (i, SubjectTarget::$variant))
        }
    };
}

macro_rules! variant_value {
    ($life:lifetime, $name:literal, $variant:ident, $value:expr) => {
        move |s: &$life str| -> IResult<&$life str, SubjectTarget> {
            tag($name)(s).map(|(i, _o)| (i, SubjectTarget::$variant($value)))
        }
    };
}

macro_rules! variant_index {
    ($life:lifetime, $name:literal, $variant:tt) => {
        move |s: &$life str| -> IResult<&$life str, SubjectTarget> {
            preceded(tag($name), parse_number)(s).map(|(i, num)| (i, SubjectTarget::$variant(num)))
        }
    };
}

// noinspection RsNeedlessLifetimes
fn parse_subject<'a>(input: &'a str) -> IResult<&'a str, Subject> {
    let (input, target) = alt((
        alt((
            variant!('a, "acc", Acceleration),
            variant_value!('a, "atc", Ats, 271),
            variant_index!('a, "ats", Ats),
            variant!('a, "locobrakecylinder", LocoBrakeCylinder),
            variant!('a, "bc", BrakeCylinder),
            variant!('a, "locobrakepipe", LocoBrakePipe),
            variant!('a, "bp", BrakePipe),
            variant!('a, "brake", Brake),
            variant!('a, "locobrake", LocoBrake),
            variant!('a, "csc", ConstSpeedSystem),
            // These 4 must come before door, as door is a prefix for them and will partially parse, then fail
            variant_index!('a, "doorl", DoorLeft),
            variant_index!('a, "doorr", DoorRight),
            variant!('a, "doorbuttonl", DoorButtonLeft),
            variant!('a, "doorbuttonr", DoorButtonRight),
            variant!('a, "door", Door),
            variant!('a, "er", EqualizingReservoir),
            variant!('a, "hour", Hour),
        )),
        alt((
            variant!('a, "kmph", KilometersPerHour),
            variant!('a, "min", Minute),
            variant!('a, "motor", MotorAcceleration),
            variant!('a, "mph", MilesPerHour),
            variant!('a, "mr", MainReservoir),
            variant!('a, "ms", MetersPerSecond),
            variant!('a, "power", PowerNotch),
            variant!('a, "rev", Reverser),
            variant!('a, "sap", StraightAirPipe),
            variant!('a, "sec", Second),
            variant!('a, "true", True),
            variant!('a, "klaxon", Klaxon),
            variant!('a, "primaryklaxon", PrimaryKlaxon),
            variant!('a, "secondaryklaxon", SecondaryKlaxon),
            variant!('a, "musicklaxon", MusicKlaxon),
            variant!('a, "passalarm", PassAlarm),
            variant!('a, "pilotlamp", PilotLamp),
            variant!('a, "stationadjustalarm", StationAdjustAlarm),
        )),
    ))(input)?;

    let (input, digit) = opt(preceded(tag("d"), parse_number))(input)?;
    Ok((input, Subject { target, digit }))
}

fn parse_number(input: &str) -> IResult<&str, u64> {
    map_res(take_while(char::is_numeric), u64::from_str)(input)
}

#[cfg(test)]
mod test {
    use super::{parse_subject, Subject, SubjectTarget};

    #[test]
    fn no_suffix() {
        let (input, output) = parse_subject("rev").expect("Failed to parse");
        assert!(input.is_empty());
        assert_eq!(
            output,
            Subject {
                target: SubjectTarget::Reverser,
                digit: None
            }
        )
    }

    #[test]
    fn no_suffix_index() {
        let (input, output) = parse_subject("ats232").expect("Failed to parse");
        assert!(input.is_empty());
        assert_eq!(
            output,
            Subject {
                target: SubjectTarget::Ats(232),
                digit: None
            }
        )
    }

    #[test]
    fn suffix() {
        let (input, output) = parse_subject("revd3").expect("Failed to parse");
        assert!(input.is_empty());
        assert_eq!(
            output,
            Subject {
                target: SubjectTarget::Reverser,
                digit: Some(3)
            }
        )
    }

    #[test]
    fn suffix_index() {
        let (input, output) = parse_subject("ats232d3").expect("Failed to parse");
        assert!(input.is_empty());
        assert_eq!(
            output,
            Subject {
                target: SubjectTarget::Ats(232),
                digit: Some(3)
            }
        )
    }

    #[test]
    fn door_parsing_order() {
        let (input, output) = parse_subject("doorl232").expect("Failed to parse");
        assert!(input.is_empty());
        assert_eq!(
            output,
            Subject {
                target: SubjectTarget::DoorLeft(232),
                digit: None
            }
        )
    }
}
