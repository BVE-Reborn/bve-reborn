use super::*;

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
                let namespace = command.namespace.clone().or(self.current_namespace.clone())?;
                let name: SmartString<LazyCompact> = command.name.chars().flat_map(char::to_lowercase).collect();
                let suffix: Option<SmartString<LazyCompact>> = command
                    .suffix
                    .as_ref()
                    .map(|s| s.chars().flat_map(char::to_lowercase).collect());

                match (namespace.as_str(), name.as_str(), suffix.as_deref()) {
                    ("test", "test", Some("test")) => true,
                    _ => None?,
                };

                unimplemented!()
            };
            if let Some(command) = parsed_command {
                return Some(command);
            }
        }
        None
    }
}
