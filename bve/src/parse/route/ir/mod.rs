use super::parser::{ArgumentSmallVec, Command};
use crate::{ColorU8RGB, ColorU8RGBA, Time};
use bve_derive::FromRouteCommand;
use smallvec::SmallVec;
use smartstring::{LazyCompact, SmartString};
use specials::*;
use std::{num::NonZeroU64, str::FromStr};

#[macro_use]
mod specials;

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

flag_enum!(OptionObjectVisibilityMode, u8, Legacy = 0, TrackBased = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionObjectVisibility {
    #[command(default = "OptionObjectVisibilityMode::Legacy")]
    pub mode: OptionObjectVisibilityMode,
}

flag_enum!(OptionSectionBehaviorMode, u8, Default = 0, Simplified = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionSectionBehavior {
    #[command(default = "OptionSectionBehaviorMode::Default")]
    pub mode: OptionSectionBehaviorMode,
}

flag_enum!(OptionCantBehaviorMode, u8, Unsigned = 0, Signed = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionCantBehavior {
    #[command(default = "OptionCantBehaviorMode::Unsigned")]
    pub mode: OptionCantBehaviorMode,
}

flag_enum!(OptionFogBehaviorMode, u8, BlockBased = 0, Interpolated = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionFogBehavior {
    #[command(default = "OptionFogBehaviorMode::BlockBased")]
    pub mode: OptionFogBehaviorMode,
}

flag_enum!(OptionCompatibleTransparencyMode, u8, Off = 0, On = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionCompatibleTransparency {
    #[command(default = "OptionCompatibleTransparencyMode::Off")]
    pub mode: OptionCompatibleTransparencyMode,
}

flag_enum!(OptionEnableBveTsHacksMode, u8, Off = 0, On = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionEnableBveTsHacks {
    #[command(default = "OptionEnableBveTsHacksMode::Off")]
    pub mode: OptionEnableBveTsHacksMode,
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
