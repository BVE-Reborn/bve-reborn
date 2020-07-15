use super::parser::{ArgumentSmallVec, Command};
use crate::{ColorU8RGB, ColorU8RGBA};
use bve_derive::FromRouteCommand;
use smallvec::SmallVec;
use std::{num::NonZeroU64, str::FromStr};

pub trait FromRouteCommand {
    fn from_route_command(command: Command<'_>) -> Option<Self>
    where
        Self: Sized;
}

pub trait FromVariadicRouteArgument<'a> {
    type Error;

    fn from_variadic_route_argument(value: &ArgumentSmallVec<'a>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl<'a, Array> FromVariadicRouteArgument<'a> for SmallVec<Array>
where
    Array: smallvec::Array,
    Array::Item: FromStr,
{
    type Error = <Array::Item as FromStr>::Err;

    fn from_variadic_route_argument(value: &ArgumentSmallVec<'a>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut out = Self::new();
        for &v in value {
            out.push(v.parse()?)
        }
        Ok(out)
    }
}

impl<'a> FromVariadicRouteArgument<'a> for ColorU8RGB {
    type Error = ();

    fn from_variadic_route_argument(value: &ArgumentSmallVec<'a>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let get = |idx: usize| value.get(idx).ok_or(())?.parse::<u8>().map_err(|_| ());
        Ok(Self::new(get(0)?, get(1)?, get(2)?))
    }
}

impl<'a> FromVariadicRouteArgument<'a> for ColorU8RGBA {
    type Error = ();

    fn from_variadic_route_argument(value: &ArgumentSmallVec<'a>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let get = |idx: usize| value.get(idx).ok_or(())?.parse::<u8>().map_err(|_| ());
        Ok(Self::new(get(0)?, get(1)?, get(2)?, get(3)?))
    }
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsUnitOfLength {
    #[command(variadic)]
    pub factors: SmallVec<[f32; 2]>,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsUnitOfSpeed {
    pub factor: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionsBlockLength {
    /// unit: UnitOfLength
    pub length: f32,
}

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

flag_enum!(OptionObjectVisibilityMode, u8, Legacy = 0, TrackBased = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionObjectVisibility {
    pub mode: OptionObjectVisibilityMode,
}

flag_enum!(OptionSectionBehaviorMode, u8, Default = 0, Simplified = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionSectionBehavior {
    pub mode: OptionSectionBehaviorMode,
}

flag_enum!(OptionCantBehaviorMode, u8, Unsigned = 0, Signed = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionCantBehavior {
    pub mode: OptionCantBehaviorMode,
}

flag_enum!(OptionFogBehaviorMode, u8, BlockBased = 0, Interpolated = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionFogBehavior {
    pub mode: OptionFogBehaviorMode,
}

flag_enum!(OptionCompatibleTransparencyMode, u8, Off = 0, On = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionCompatibleTransparency {
    pub mode: OptionCompatibleTransparencyMode,
}

flag_enum!(OptionEnableBveTsHacksMode, u8, Off = 0, On = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionEnableBveTsHacks {
    pub mode: OptionEnableBveTsHacksMode,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteComment {
    pub comment: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteImage {
    pub file: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteTimetable {
    pub text: String,
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
    pub text: RouteChangeMode,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteGauge {
    /// unit: mm
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
    pub gravity: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteElevation {
    /// unit: UnitOfLength
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteTemperature {
    /// unit: celsius
    pub temperature: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RoutePressure {
    /// unit: kPa
    pub pressure: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteDisplaySpeed {
    pub unit: String,
    /// Conversion factor from km/h -> this
    pub factor: f32,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteLoadingScreen {
    pub image: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteStartTime {
    // TODO: Actually implement time
    pub time: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteDynamicLight {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteAmbientLight {
    #[command(variadic)]
    pub color: ColorU8RGB,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteDirectionalLight {
    #[command(variadic)]
    pub color: ColorU8RGB,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteLightDirection {
    pub theta: f32,
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
    pub view: RouteInitialViewpointMode,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteDeveloperId {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrainFolder {
    pub folder: String,
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
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrainTimetable {
    #[command(index)]
    pub timetable_index: u64,
    #[command(suffix)]
    pub timetable_suffix: TimetableSuffix,
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrainVelocity {
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
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct StructurePole {
    #[command(index)]
    pub number_of_additional_rails: u64,
    #[command(index)]
    pub pole_structure_index: u64,
    pub file_name: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TextureBackgroundLoad {
    #[command(index)]
    pub background_texture_index: u64,
    pub file_name: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TextureBackgroundX {
    #[command(index)]
    pub background_texture_index: u64,
    pub repetition_count: u64,
}

flag_enum!(TextureBackgroundAspectMode, u8, Fixed = 0, Aspect = 1);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TextureBackgroundAspect {
    #[command(index)]
    pub background_texture_index: u64,
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
    pub signal_file: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct SignalSplit {
    #[command(index)]
    pub signal_index: u64,
    pub signal_file: String,
    pub glow_file: String,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackRailStart {
    pub rail_index: NonZeroU64,
    /// unit: UnitOfDistance
    pub x_offset: f32,
    /// unit: UnitOfDistance
    pub y_offset: f32,
    pub rail_type: u64,
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
    pub rail_index: NonZeroU64,
    pub rail_type: u64,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct TrackRailEnd {
    pub rail_index: NonZeroU64,
    /// unit: UnitOfDistance
    pub x_offset: f32,
    /// unit: UnitOfDistance
    pub y_offset: f32,
}
