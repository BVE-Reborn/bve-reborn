use super::*;

macro_rules! command_match {
    ($command:expr, $ns:expr, $name:expr, $suffix:expr, $($($pat:pat)|+ $(=> $variant:ident)? $(=|> $expression:expr)?),* $(,)?) => {
        match ($ns, $name, $suffix) {$(
            $($pat)|+ => $(ParsedCommand::$variant($variant::from_route_command($command)?))? $($expression)?,
        )*
             _ => None?,
        }
    };
}

struct CommandParserIterator<T>
where
    T: Iterator<Item = Command>,
{
    current_namespace: Option<SmartString<LazyCompact>>,
    instruction_stream: T,
}
impl<T> Iterator for CommandParserIterator<T>
where
    T: Iterator<Item = Command>,
{
    type Item = ParsedCommand;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(command) = self.instruction_stream.next() {
            let parsed_command: Option<ParsedCommand> = try {
                let namespace = command.namespace.clone().or_else(|| self.current_namespace.clone())?;
                let name: SmartString<LazyCompact> = command.name.chars().flat_map(char::to_lowercase).collect();
                let suffix: Option<SmartString<LazyCompact>> = command
                    .suffix
                    .as_ref()
                    .map(|s| s.chars().flat_map(char::to_lowercase).collect());

                command_match!(command, namespace.as_str(), name.as_str(), suffix.as_deref(),
                    ("options", "unitoflength", _) => OptionsUnitOfLength,
                    ("options", "unitofspeed", _) => OptionsUnitOfSpeed,
                    ("options", "blocklength", _) => OptionsBlockLength,
                    ("options", "objectvisibility", _) => OptionsObjectVisibility,
                    ("options", "sectionbehavior", _) => OptionsSectionBehavior,
                    ("options", "cantbehavior", _) => OptionsCantBehavior,
                    ("options", "fogbehavior", _) => OptionsFogBehavior,
                    ("options", "compatibletransparencymode", _) => OptionsCompatibleTransparency,
                    ("options", "enablebvetshacks", _) => OptionsEnableBveTsHacks,
                    ("route", "comment", _) => RouteComment,
                    ("route", "image", _) => RouteImage,
                    ("route", "timetable", _) => RouteTimetable,
                    ("route", "change", _) => RouteChange,
                    ("route", "gauge", _) | ("train", "gauge", _) => RouteGauge,
                    ("route", "signal", _) => RouteSignal,
                    ("route", "runinterval", _) | ("train", "interval", _) => RouteRunInterval,
                    ("route", "accelerationduetogravity", _) => RouteAccelerationDueToGravity,
                    ("route", "elevation", _) => RouteElevation,
                    ("route", "temperature", _) => RouteTemperature,
                    ("route", "pressure", _) => RoutePressure,
                    ("route", "displayspeed", _) => RouteDisplaySpeed,
                    ("route", "loadingscreen", _) => RouteLoadingScreen,
                    ("route", "starttime", _) => RouteStartTime,
                    ("route", "dynamiclight", _) => RouteDynamicLight,
                    ("route", "ambientlight", _) => RouteAmbientLight,
                    ("route", "directionallight", _) => RouteDirectionalLight,
                    ("route", "lightdirection", _) => RouteLightDirection,
                    ("route", "initialviewpoint", _) => RouteInitialViewpoint,
                    ("route", "developerid", _) => RouteDeveloperId,
                    ("train", "folder" , _) |("train", "file", _) => TrainFolder,
                    ("train", "run", _) | ("train", "rail", _) => TrainRail,
                    ("train", "flange", _) => TrainFlange,
                    ("train", "timetable", Some("day" | "night")) => TrainTimetable,
                    ("train", "velocity", _) => TrainVelocity,
                    ("structure", "pole", _) => StructurePole,
                    ("structure", command_name, _) =|> {
                        let mut parsed = StructureCommand::from_route_command(command)?;
                        parsed.command = Some(match command_name {
                            "ground" => StructureCommandKind::Ground,
                            "rail" => StructureCommandKind::Rail,
                            "walll" => StructureCommandKind::WallL,
                            "wallr" => StructureCommandKind::WallR,
                            "dikel" => StructureCommandKind::DikeL,
                            "diker" => StructureCommandKind::DikeR,
                            "forml" => StructureCommandKind::FormL,
                            "formr" => StructureCommandKind::FormR,
                            "formcl" => StructureCommandKind::FormCL,
                            "formcr" => StructureCommandKind::FormCR,
                            "roofl" => StructureCommandKind::RoofL,
                            "roofr" => StructureCommandKind::RoofR,
                            "roofcl" => StructureCommandKind::RoofCL,
                            "roofcr" => StructureCommandKind::RoofCR,
                            "crackl" => StructureCommandKind::CrackL,
                            "crackr" => StructureCommandKind::CrackR,
                            "freeobj" => StructureCommandKind::FreeObj,
                            "beacon" => StructureCommandKind::Beacon,
                            _ => None?,
                        });
                        ParsedCommand::StructureCommand(parsed)
                    },
                    ("texture", "background", suffix) =|> {
                        match suffix {
                            None | Some("load") => ParsedCommand::TextureBackgroundLoad(TextureBackgroundLoad::from_route_command(command)?),
                            Some("x") => ParsedCommand::TextureBackgroundX(TextureBackgroundX::from_route_command(command)?),
                            Some("aspect") => ParsedCommand::TextureBackgroundAspect(TextureBackgroundAspect::from_route_command(command)?),
                            _ => None?,
                        }
                    },
                    ("cycle", "ground", _) => CycleGround,
                    ("cycle", "rail", _) => CycleRail,
                    // TODO: Fix signal parsing
                    ("", "signal", _) =|> {
                        match command.arguments.len() {
                            0 => None?,
                            1 => ParsedCommand::SignalSingle(SignalSingle::from_route_command(command)?),
                            _ => ParsedCommand::SignalSplit(SignalSplit::from_route_command(command)?),
                        }
                    },
                    ("track", "railstart", _) => TrackRailStart,
                    ("track", "rail", _) => TrackRail,
                    ("track", "railtype", _) => TrackRailType,
                    ("track", "railend", _) => TrackRailEnd,
                    ("track", "accuracy", _) => TrackAccuracy,
                    ("track", "adhesion", _) => TrackAdhesion,
                    ("track", "pitch", _) => TrackPitch,
                    ("track", "curve", _) => TrackCurve,
                    ("track", "turn", _) => TrackTurn,
                    ("track", "height", _) => TrackHeight,
                    ("track", "freeobj", _) => TrackFreeObj,
                    ("track", "wall", _) => TrackWall,
                    ("track", "wallend", _) => TrackWallEnd,
                    ("track", "dike", _) => TrackDike,
                    ("track", "dikeend", _) => TrackDikeEnd,
                    ("track", "pole", _) => TrackPole,
                    ("track", "poleend", _) => TrackPoleEnd,
                    ("track", "crack", _) => TrackCrack,
                    ("track", "ground", _) => TrackGround,
                    ("track", "sta", _) => TrackSta,
                    ("track", "station", _) =|> ParsedCommand::TrackSta(TrackStation::from_route_command(command)?.into()),
                    ("track", "stop", _) => TrackStop,
                    ("track", "form", _) => TrackForm,
                    ("track", "limit", _) => TrackLimit,
                    ("track", "section", _) => TrackSection,
                    ("track", "sigf", _) => TrackSigF,
                    ("track", "signal", _) | ("track", "sig", _) => TrackSignal,
                    ("track", "relay", _) => TrackRelay,
                    ("track", "beacon", _) => TrackBeacon,
                    ("track", "tr", _) | ("track", "transponder", _) => TrackTransponder,
                    ("track", "atssn", _) =|> {
                        ParsedCommand::TrackTransponder(TrackTransponder {
                            typ: TrackTransponderType::SType,
                            signal: 0,
                            switch_system: TrackTransponderSwitchSystem::Switch,
                            x_offset: 0.0,
                            y_offset: 0.0,
                            yaw: 0.0,
                            pitch: 0.0,
                            roll: 0.0
                        })
                    },
                    ("track", "atsp", _) =|> {
                        ParsedCommand::TrackTransponder(TrackTransponder {
                            typ: TrackTransponderType::AtsPPaternRenewal,
                            signal: 0,
                            switch_system: TrackTransponderSwitchSystem::Switch,
                            x_offset: 0.0,
                            y_offset: 0.0,
                            yaw: 0.0,
                            pitch: 0.0,
                            roll: 0.0
                        })
                    },
                    ("track", "pattern", _) => TrackPattern,
                    ("track", "plimit", _) => TrackPLimit,
                    ("track", "back", _) => TrackBack,
                    ("track", "fog", _) => TrackFog,
                    ("track", "brightness", _) => TrackBrightness,
                    ("track", "marker", _) =|> {
                        match command.arguments.len() {
                            0 => None?,
                            1 => ParsedCommand::TrackMarkerXml(TrackMarkerXml::from_route_command(command)?),
                            _ => ParsedCommand::TrackMarker(TrackMarker::from_route_command(command)?),
                        }
                    },
                    ("track", "textmarker", _) => TrackTextMarker,
                    ("track", "poi", _) | ("track", "pointofinterest", _) => TrackPointOfInterest,
                    ("track", "pretrain", _) => TrackPreTrain,
                    ("track", "announce", _) => TrackAnnounce,
                    ("track", "doppler", _) => TrackDoppler,
                    ("track", "buffer", _) => TrackBuffer,
                    ("track", "destination", _) => TrackDestination,
                )
            };

            if let Some(command) = parsed_command {
                return Some(command);
            }
        }
        None
    }
}
