use super::parser::{ArgumentSmallVec, Command};
use crate::{ColorU8RGB, ColorU8RGBA};
use bve_derive::FromRouteCommand;
use smallvec::SmallVec;
use std::str::FromStr;

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

macro_rules! binary_flag_enum {
    ($name:ident, $zero:ident, $one:ident) => {
        #[derive(Debug, Clone, PartialEq)]
        #[repr(u8)]
        pub enum $name {
            $zero = 0,
            $one = 1,
        }
        impl FromStr for $name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.parse::<u8>().map_err(|_| ())? {
                    0 => Ok(Self::$zero),
                    1 => Ok(Self::$one),
                    _ => Err(()),
                }
            }
        }
    };
}

binary_flag_enum!(OptionObjectVisibilityMode, Legacy, TrackBased);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionObjectVisibility {
    pub mode: OptionObjectVisibilityMode,
}

binary_flag_enum!(OptionSectionBehaviorMode, Default, Simplified);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionSectionBehavior {
    pub mode: OptionSectionBehaviorMode,
}

binary_flag_enum!(OptionCantBehaviorMode, Unsigned, Signed);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionCantBehavior {
    pub mode: OptionCantBehaviorMode,
}

binary_flag_enum!(OptionFogBehaviorMode, BlockBased, Interpolated);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionFogBehavior {
    pub mode: OptionFogBehaviorMode,
}

binary_flag_enum!(OptionCompatibleTransparencyMode, Off, On);
#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct OptionCompatibleTransparency {
    pub mode: OptionCompatibleTransparencyMode,
}

binary_flag_enum!(OptionEnableBveTsHacksMode, Off, On);
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

#[derive(Debug, Clone, PartialEq)]
#[repr(i8)]
pub enum RouteChangeMode {
    SafetyActiveService = -1,
    SafetyActiveEmergency = 0,
    SafetyInactiveEmergency = 1,
}
impl FromStr for RouteChangeMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i8>().map_err(|_| ())? {
            -1 => Ok(Self::SafetyActiveService),
            0 => Ok(Self::SafetyActiveEmergency),
            1 => Ok(Self::SafetyInactiveEmergency),
            _ => Err(()),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum RouteInitialViewpointMode {
    Cab = 0,
    TrackCamera = 1,
    FlybyCamera = 2,
    FlybyZoomingCamera = 3,
}
impl FromStr for RouteInitialViewpointMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u8>().map_err(|_| ())? {
            0 => Ok(Self::Cab),
            1 => Ok(Self::TrackCamera),
            2 => Ok(Self::FlybyCamera),
            3 => Ok(Self::FlybyZoomingCamera),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteInitialViewpoint {
    pub view: RouteInitialViewpointMode,
}

#[derive(Debug, Clone, PartialEq, FromRouteCommand)]
pub struct RouteDeveloperId {
    pub id: String,
}

#[derive(FromRouteCommand)]
pub struct Pole {
    #[command(index)]
    pub number_of_additional_rails: u64,
    #[command(index)]
    pub pole_structure_idx: u64,
    pub file_name: String,
}
