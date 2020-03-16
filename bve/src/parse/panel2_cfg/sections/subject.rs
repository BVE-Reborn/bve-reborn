use nom::IResult;

pub struct Subject {
    pub target: SubjectTarget,
    pub digit: u64,
}

pub enum SubjectTarget {
    Acceleration,
    Ats(i64),
    LocoBrakeCylinder,
    BrakeCylinder,
    LocoBrakePipe,
    BrakePipe,
    Brake,
    LocoBrake,
    ConstSpeedSystem,
    Door,
    DoorLeft(i64),
    DoorRight(i64),
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

pub fn parse_subject(input: &str) -> IResult<&str, Subject> {
    unimplemented!()
}
