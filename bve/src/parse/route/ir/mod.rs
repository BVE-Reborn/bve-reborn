use super::parser::{ArgumentSmallVec, Command};
use crate::{ColorU8RGB, ColorU8RGBA, Time};
use bve_derive::FromRouteCommand;
use smallvec::SmallVec;
use smartstring::{LazyCompact, SmartString};
pub use specials::*;
use std::{num::NonZeroU64, str::FromStr};

#[macro_use]
mod specials;
mod dispatch;

enum ParsedCommand {
    OptionsUnitOfLength(OptionsUnitOfLength),
    OptionsUnitOfSpeed(OptionsUnitOfSpeed),
    OptionsBlockLength(OptionsBlockLength),
    OptionsObjectVisibility(OptionsObjectVisibility),
    OptionsSectionBehavior(OptionsSectionBehavior),
    OptionsCantBehavior(OptionsCantBehavior),
    OptionsFogBehavior(OptionsFogBehavior),
    OptionsCompatibleTransparency(OptionsCompatibleTransparency),
    OptionsEnableBveTsHacks(OptionsEnableBveTsHacks),
    RouteComment(RouteComment),
    RouteImage(RouteImage),
    RouteTimetable(RouteTimetable),
    RouteChange(RouteChange),
    RouteGauge(RouteGauge),
    RouteSignal(RouteSignal),
    RouteRunInterval(RouteRunInterval),
    RouteAccelerationDueToGravity(RouteAccelerationDueToGravity),
    RouteElevation(RouteElevation),
    RouteTemperature(RouteTemperature),
    RoutePressure(RoutePressure),
    RouteDisplaySpeed(RouteDisplaySpeed),
    RouteLoadingScreen(RouteLoadingScreen),
    RouteStartTime(RouteStartTime),
    RouteDynamicLight(RouteDynamicLight),
    RouteAmbientLight(RouteAmbientLight),
    RouteDirectionalLight(RouteDirectionalLight),
    RouteLightDirection(RouteLightDirection),
    RouteInitialViewpoint(RouteInitialViewpoint),
    RouteDeveloperId(RouteDeveloperId),
    TrainFolder(TrainFolder),
    TrainRail(TrainRail),
    TrainFlange(TrainFlange),
    TrainTimetable(TrainTimetable),
    TrainVelocity(TrainVelocity),
    StructureCommand(StructureCommand),
    StructurePole(StructurePole),
    TextureBackgroundLoad(TextureBackgroundLoad),
    TextureBackgroundX(TextureBackgroundX),
    TextureBackgroundAspect(TextureBackgroundAspect),
    CycleGround(CycleGround),
    CycleRail(CycleRail),
    SignalSingle(SignalSingle),
    SignalSplit(SignalSplit),
    TrackRailStart(TrackRailStart),
    TrackRail(TrackRail),
    TrackRailType(TrackRailType),
    TrackRailEnd(TrackRailEnd),
    TrackAccuracy(TrackAccuracy),
    TrackAdhesion(TrackAdhesion),
    TrackPitch(TrackPitch),
    TrackCurve(TrackCurve),
    TrackTurn(TrackTurn),
    TrackHeight(TrackHeight),
    TrackFreeObj(TrackFreeObj),
    TrackWall(TrackWall),
    TrackWallEnd(TrackWallEnd),
    TrackDike(TrackDike),
    TrackDikeEnd(TrackDikeEnd),
    TrackPole(TrackPole),
    TrackPoleEnd(TrackPoleEnd),
    TrackCrack(TrackCrack),
    TrackGround(TrackGround),
    TrackSta(TrackSta),
    TrackStop(TrackStop),
    TrackForm(TrackForm),
    TrackLimit(TrackLimit),
    TrackSection(TrackSection),
    TrackSigF(TrackSigF),
    TrackSignal(TrackSignal),
    TrackRelay(TrackRelay),
    TrackBeacon(TrackBeacon),
    TrackTransponder(TrackTransponder),
    TrackPattern(TrackPattern),
    TrackPLimit(TrackPLimit),
    TrackBack(TrackBack),
    TrackFog(TrackFog),
    TrackBrightness(TrackBrightness),
    TrackMarker(TrackMarker),
    TrackMarkerXml(TrackMarkerXml),
    TrackTextMarker(TrackTextMarker),
    TrackPointOfInterest(TrackPointOfInterest),
    TrackPreTrain(TrackPreTrain),
    TrackAnnounce(TrackAnnounce),
    TrackDoppler(TrackDoppler),
    TrackBuffer(TrackBuffer),
    TrackDestination(TrackDestination),
}

pub trait FromRouteCommand {
    fn from_route_command(command: Command) -> Option<Self>
    where
        Self: Sized;
}

pub trait FromVariadicRouteArgument<'a> {
    type Error;

    fn from_variadic_route_argument(value: &ArgumentSmallVec) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl<'a, Array> FromVariadicRouteArgument<'a> for SmallVec<Array>
where
    Array: smallvec::Array,
    Array::Item: FromStr,
{
    type Error = <Array::Item as FromStr>::Err;

    fn from_variadic_route_argument(value: &ArgumentSmallVec) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut out = Self::new();
        for v in value {
            out.push(v.parse()?)
        }
        Ok(out)
    }
}

impl<'a> FromVariadicRouteArgument<'a> for ColorU8RGB {
    type Error = ();

    fn from_variadic_route_argument(value: &ArgumentSmallVec) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let get = |idx: usize| value.get(idx).ok_or(())?.parse::<u8>().map_err(|_| ());
        Ok(Self::new(get(0)?, get(1)?, get(2)?))
    }
}

impl<'a> FromVariadicRouteArgument<'a> for ColorU8RGBA {
    type Error = ();

    fn from_variadic_route_argument(value: &ArgumentSmallVec) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let get = |idx: usize| value.get(idx).ok_or(())?.parse::<u8>().map_err(|_| ());
        Ok(Self::new(get(0)?, get(1)?, get(2)?, get(3)?))
    }
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsUnitOfLength {
    #[command(variadic, default = "SmallVec::from_slice(&[1.0])")]
    pub factors: SmallVec<[f32; 2]>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsUnitOfSpeed {
    #[command(default = "1.0")]
    pub factor: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsBlockLength {
    /// unit: UnitOfLength
    #[command(default = "25.0")]
    pub length: f32,
}

flag_enum!(OptionsObjectVisibilityMode, u8, Legacy = 0, TrackBased = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsObjectVisibility {
    #[command(default = "OptionsObjectVisibilityMode::Legacy")]
    pub mode: OptionsObjectVisibilityMode,
}

flag_enum!(OptionsSectionBehaviorMode, u8, Default = 0, Simplified = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsSectionBehavior {
    #[command(default = "OptionsSectionBehaviorMode::Default")]
    pub mode: OptionsSectionBehaviorMode,
}

flag_enum!(OptionsCantBehaviorMode, u8, Unsigned = 0, Signed = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsCantBehavior {
    #[command(default = "OptionsCantBehaviorMode::Unsigned")]
    pub mode: OptionsCantBehaviorMode,
}

flag_enum!(OptionsFogBehaviorMode, u8, BlockBased = 0, Interpolated = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsFogBehavior {
    #[command(default = "OptionsFogBehaviorMode::BlockBased")]
    pub mode: OptionsFogBehaviorMode,
}

flag_enum!(OptionsCompatibleTransparencyMode, u8, Off = 0, On = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsCompatibleTransparency {
    #[command(default = "OptionsCompatibleTransparencyMode::Off")]
    pub mode: OptionsCompatibleTransparencyMode,
}

flag_enum!(OptionsEnableBveTsHacksMode, u8, Off = 0, On = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsEnableBveTsHacks {
    #[command(default = "OptionsEnableBveTsHacksMode::Off")]
    pub mode: OptionsEnableBveTsHacksMode,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteComment {
    pub comment: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteImage {
    pub file: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteTimetable {
    pub text: SmartString<LazyCompact>,
}

flag_enum!(
    RouteChangeMode,
    i8,
    SafetyActiveService = -1,
    SafetyActiveEmergency = 0,
    SafetyInactiveEmergency = 1
);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteChange {
    #[command(default = "RouteChangeMode::SafetyActiveEmergency")]
    pub text: RouteChangeMode,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteGauge {
    /// unit: mm
    #[command(default = "1435.0")]
    pub gauge: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteSignal {
    #[command(index)]
    pub aspect_index: u8,
    /// unit: UnitOfSpeed
    pub speed: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteRunInterval {
    /// unit: s
    #[command(variadic)]
    pub intervals: SmallVec<[f32; 4]>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteAccelerationDueToGravity {
    /// unit: m/s^2
    #[command(default = "9.80665")]
    pub gravity: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteElevation {
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteTemperature {
    /// unit: celsius
    #[command(default = "20.0")]
    pub temperature: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RoutePressure {
    /// unit: kPa
    #[command(default = "101.325")]
    pub pressure: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteDisplaySpeed {
    #[command(default = "SmartString::default()")]
    pub unit: SmartString<LazyCompact>,
    /// Conversion factor from km/h -> this
    pub factor: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteLoadingScreen {
    pub image: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteStartTime {
    pub time: Time,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteDynamicLight {
    pub path: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteAmbientLight {
    #[command(variadic, default = "ColorU8RGB::new(160, 160, 160)")]
    pub color: ColorU8RGB,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteDirectionalLight {
    #[command(variadic, default = "ColorU8RGB::new(160, 160, 160)")]
    pub color: ColorU8RGB,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteLightDirection {
    #[command(default = "60.0")]
    pub theta: f32,
    #[command(default = "26.57")]
    pub phi: f32,
}

flag_enum!(
    RouteInitialViewpointMode,
    u8,
    Cab = 0,
    TrackCamera = 1,
    FlybyCamera = 2,
    FlybyZoomingCamera = 3
);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteInitialViewpoint {
    #[command(default = "RouteInitialViewpointMode::Cab")]
    pub view: RouteInitialViewpointMode,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteDeveloperId {
    pub id: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrainFolder {
    pub folder: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrainRail {
    #[command(index)]
    pub rail_type_index: u64,
    pub run_sound_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrainFlange {
    #[command(index)]
    pub rail_type_index: u64,
    pub flange_sound_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrainTimetable {
    #[command(index)]
    pub timetable_index: u64,
    #[command(suffix)]
    pub timetable_suffix: TimetableSuffix,
    pub filename: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrainVelocity {
    #[command(default = "0.0")]
    /// unit: UnitOfSpeed
    pub max_ai_speed: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructureCommandKind {
    Ground,
    Rail,
    WallL,
    WallR,
    DikeL,
    DikeR,
    FormL,
    FormR,
    FormCL,
    FormCR,
    RoofL,
    RoofR,
    RoofCL,
    RoofCR,
    CrackL,
    CrackR,
    FreeObj,
    Beacon,
}
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct StructureCommand {
    #[command(ignore)]
    pub command: Option<StructureCommandKind>,
    #[command(index)]
    pub structure_index: u64,
    pub filename: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct StructurePole {
    #[command(index)]
    pub number_of_additional_rails: u64,
    #[command(index)]
    pub pole_structure_index: u64,
    pub file_name: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TextureBackgroundLoad {
    #[command(index)]
    pub background_texture_index: u64,
    pub file_name: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TextureBackgroundX {
    #[command(index)]
    pub background_texture_index: u64,
    #[command(default = "6")]
    pub repetition_count: u64,
}

flag_enum!(TextureBackgroundAspectMode, u8, Fixed = 0, Aspect = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TextureBackgroundAspect {
    #[command(index)]
    pub background_texture_index: u64,
    #[command(default = "TextureBackgroundAspectMode::Fixed")]
    pub mode: TextureBackgroundAspectMode,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct CycleGround {
    #[command(index)]
    pub ground_structure_index: u64,
    #[command(variadic)]
    pub ground_structures: SmallVec<[u64; 8]>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct CycleRail {
    #[command(index)]
    pub rail_structure_index: u64,
    #[command(variadic)]
    pub rail_structures: SmallVec<[u64; 8]>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct SignalSingle {
    #[command(index)]
    pub signal_index: u64,
    pub signal_file: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct SignalSplit {
    #[command(index)]
    pub signal_index: u64,
    pub signal_file: SmartString<LazyCompact>,
    #[command(default = "SmartString::default()")]
    pub glow_file: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackRailStart {
    pub rail_index: NonZeroU64,
    /// unit: UnitOfDistance
    #[command(optional)]
    pub x_offset: Option<f32>,
    /// unit: UnitOfDistance
    #[command(optional)]
    pub y_offset: Option<f32>,
    #[command(optional)]
    pub rail_type: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackRail {
    pub rail_index: NonZeroU64,
    /// unit: UnitOfDistance
    pub x_offset: f32,
    /// unit: UnitOfDistance
    pub y_offset: f32,
    pub rail_type: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackRailType {
    #[command(default = "0")]
    pub rail_index: u64,
    #[command(default = "0")]
    pub rail_type: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackRailEnd {
    pub rail_index: NonZeroU64,
    /// unit: UnitOfDistance
    #[command(optional)]
    pub x_offset: Option<f32>,
    /// unit: UnitOfDistance
    #[command(optional)]
    pub y_offset: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackAccuracy {
    #[command(default = "2.0")]
    pub accuracy: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackAdhesion {
    #[command(default = "100.0")]
    pub accuracy: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackPitch {
    #[command(default = "0.0")]
    pub accuracy: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackCurve {
    /// unit: UnitOfDistance
    #[command(default = "0.0")]
    pub curve: f32,
    /// unit: Millimeters
    #[command(default = "0.0")]
    pub cant: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackTurn {
    #[command(default = "0.0")]
    pub turn: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackHeight {
    /// unit: UnitOfDistance
    #[command(default = "0.0")]
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackFreeObj {
    #[command(default = "0")]
    pub rail_index: u64,
    #[command(default = "0")]
    pub structure_index: u64,
    /// unit: UnitOfDistance
    #[command(default = "0.0")]
    pub x_offset: f32,
    /// unit: UnitOfDistance
    #[command(default = "0.0")]
    pub y_offset: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub yaw: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub pitch: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub roll: f32,
}

flag_enum!(StructureDirection, i8, Left = -1, Both = 0, Right = 1);

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackWall {
    #[command(default = "0")]
    pub rail_index: u64,
    #[command(default = "StructureDirection::Both")]
    pub direction: StructureDirection,
    #[command(default = "0")]
    pub structure_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackWallEnd {
    #[command(default = "0")]
    pub rail_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackDike {
    #[command(default = "0")]
    pub rail_index: u64,
    #[command(default = "StructureDirection::Both")]
    pub direction: StructureDirection,
    #[command(default = "0")]
    pub structure_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackDikeEnd {
    #[command(default = "0")]
    pub rail_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackPole {
    #[command(default = "0")]
    pub rail_index: u64,
    #[command(default = "0")]
    pub number_of_additional_rails: u64,
    #[command(default = "0")]
    pub location: u64,
    #[command(default = "0")]
    pub interval: u64,
    #[command(default = "0")]
    pub structure_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackPoleEnd {
    #[command(default = "0")]
    pub rail_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackCrack {
    #[command(default = "0")]
    pub rail_index1: u64,
    #[command(default = "0")]
    pub rail_index2: u64,
    #[command(default = "0")]
    pub structure_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackGround {
    #[command(default = "0")]
    pub cycle_index: u64,
}

flag_enum!(PassAlarmMode, u8, Silent = 0, Enabled = 1);
flag_enum!(ForcedRedSingleMode, u8, Unaffected = 0, Enabled = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackSta {
    #[command(default = "SmartString::new()")]
    pub name: SmartString<LazyCompact>,
    pub arrival_time: ArrivalTimeState,
    pub departure_time: DepartureTimeState,
    #[command(default = "PassAlarmMode::Silent")]
    pub pass_alarm: PassAlarmMode,
    #[command(default = "StationDoorMode::None")]
    pub doors: StationDoorMode,
    #[command(default = "ForcedRedSingleMode::Unaffected")]
    pub forced_red_signal: ForcedRedSingleMode,
    #[command(default = "SystemAtsMode::ATS")]
    pub system: SystemAtsMode,
    #[command(default = "SmartString::new()")]
    pub arrival_sound: SmartString<LazyCompact>,
    #[command(default = "15.0")]
    pub stop_duration: f32,
    #[command(default = "100.0")]
    pub passenger_ratio: f32,
    #[command(default = "SmartString::new()")]
    pub departure_sound: SmartString<LazyCompact>,
    #[command(optional)]
    pub timetable_index: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackStation {
    #[command(default = "SmartString::new()")]
    pub name: SmartString<LazyCompact>,
    pub arrival_time: ArrivalTimeState,
    pub departure_time: DepartureTimeState,
    #[command(default = "ForcedRedSingleMode::Unaffected")]
    pub forced_red_signal: ForcedRedSingleMode,
    #[command(default = "SystemAtsMode::ATS")]
    pub system: SystemAtsMode,
    #[command(default = "SmartString::new()")]
    pub departure_sound: SmartString<LazyCompact>,
}

impl From<TrackStation> for TrackSta {
    fn from(station: TrackStation) -> Self {
        Self {
            name: station.name,
            arrival_time: station.arrival_time,
            departure_time: station.departure_time,
            pass_alarm: PassAlarmMode::Silent,
            doors: StationDoorMode::Both,
            forced_red_signal: station.forced_red_signal,
            system: station.system,
            arrival_sound: SmartString::default(),
            stop_duration: 15.0,
            passenger_ratio: 100.0,
            departure_sound: station.departure_sound,
            timetable_index: None,
        }
    }
}

flag_enum!(SignPostDirection, i8, Left = -1, None = 0, Right = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackStop {
    #[command(default = "SignPostDirection::None")]
    pub direction: SignPostDirection,
    /// unit: UnitOfLength
    #[command(default = "5.0")]
    pub backwards_tolerance: f32,
    /// unit: UnitOfLength
    #[command(default = "5.0")]
    pub forwards_tolerance: f32,
    #[command(default = "0")]
    pub cars: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackForm {
    pub rail_index1: u64,
    pub rail_index2: FormRailIndex2Data,
    pub structure_index: u64,
    pub form_structure_index: u64,
}

flag_enum!(TrackLimitPostDirection, i8, Left = -1, None = 0, Right = 1);
flag_enum!(TrackLimitCourceDirection, i8, Left = -1, None = 0, Right = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackLimit {
    /// unit: UnitOfSpeed
    #[command(default = "0.0")]
    pub speed: f32,
    #[command(default = "TrackLimitPostDirection::None")]
    pub post: TrackLimitPostDirection,
    #[command(default = "TrackLimitCourceDirection::None")]
    pub cource: TrackLimitCourceDirection,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackSection {
    #[command(variadic)]
    pub sections: SmallVec<[u64; 4]>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackSigF {
    pub signal_index: u64,
    pub section: u64,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub x_offset: f32,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub y_offset: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub yaw: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub pitch: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub roll: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackSignal {
    #[command(default = "-2")]
    pub typ: i64,
    #[command(default = "SmartString::new()")]
    pub ignore: SmartString<LazyCompact>,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub x_offset: f32,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub y_offset: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub yaw: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub pitch: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub roll: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackRelay {
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub x_offset: f32,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub y_offset: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub yaw: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub pitch: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub roll: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackBeacon {
    pub typ: u64,
    pub structure_index: i64,
    #[command(default = "0")]
    pub section: i64,
    #[command(default = "0")]
    pub data: i64,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub x_offset: f32,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub y_offset: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub yaw: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub pitch: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub roll: f32,
}

flag_enum!(
    TrackTransponderType,
    u8,
    SType = 0,
    SNType = 1,
    AccidentialDeparture = 2,
    AtsPPaternRenewal = 3,
    AtsPImmediateStop = 4
);
flag_enum!(TrackTransponderSwitchSystem, i8, DoNothing = -1, Switch = 0);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackTransponder {
    #[command(default = "TrackTransponderType::SType")]
    pub typ: TrackTransponderType,
    #[command(default = "0")]
    pub signal: u64,
    #[command(default = "TrackTransponderSwitchSystem::Switch")]
    pub switch_system: TrackTransponderSwitchSystem,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub x_offset: f32,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub y_offset: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub yaw: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub pitch: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub roll: f32,
}

flag_enum!(TrackPatternType, u8, Temporary = 0, Permanent = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackPattern {
    pub typ: TrackPatternType,
    /// unit: UnitOfSpeed
    pub speed: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackPLimit {
    /// unit: UnitOfSpeed
    pub speed: f32,
}

impl From<TrackPLimit> for TrackPattern {
    fn from(other: TrackPLimit) -> Self {
        Self {
            typ: TrackPatternType::Permanent,
            speed: other.speed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackBack {
    pub background_texture_index: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackFog {
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub starting_distance: f32,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub ending_distance: f32,
    #[command(default = "128")]
    pub red: u8,
    #[command(default = "128")]
    pub green: u8,
    #[command(default = "128")]
    pub blue: u8,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackBrightness {
    #[command(default = "255")]
    pub brightness: u8,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackMarker {
    pub filename: SmartString<LazyCompact>,
    #[command(default = "0.0")]
    pub display_distance: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackMarkerXml {
    pub filename: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackTextMarker {
    pub text: SmartString<LazyCompact>,
    #[command(default = "0.0")]
    pub display_distance: f32,
    pub color: TextMarkerColor,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackPointOfInterest {
    pub rail_index: u64,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub x_offset: f32,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub y_offset: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub yaw: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub pitch: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub roll: f32,
    pub text: SmartString<LazyCompact>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackPreTrain {
    pub time: Time,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackAnnounce {
    pub filename: SmartString<LazyCompact>,
    /// unit: UnitOfSpeed
    #[command(default = "0.0")]
    pub speed: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackDoppler {
    pub filename: SmartString<LazyCompact>,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub x_offset: f32,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub y_offset: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackBuffer;

flag_enum!(TrackDestinationType, i8, AiOnly = -1, All = 0, PlayerOnly = 1);
flag_enum!(TrackDestinationTriggerOnce, u8, All = 0, Once = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackDestination {
    pub typ: TrackDestinationType,
    pub beacon_structure_index: i64,
    pub next_destination: i64,
    pub previous_destination: i64,
    pub trigger_once: TrackDestinationTriggerOnce,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub x_offset: f32,
    /// unit: UnitOfLength
    #[command(default = "0.0")]
    pub y_offset: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub yaw: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub pitch: f32,
    /// unit: Degrees
    #[command(default = "0.0")]
    pub roll: f32,
}
