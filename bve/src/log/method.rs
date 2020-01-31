use crate::log::Message;
use std::io::Write;

#[derive(Clone, Debug)]
pub enum SerializationMethod {
    Bincode,
    Json,
    JsonPretty,
}

impl SerializationMethod {
    pub fn serialize(&self, writer: &mut impl Write, message: &Message) {
        match self {
            Self::Bincode => bincode::serialize_into(writer, message).expect("Bincode serialization failed"),
            Self::Json => {
                serde_json::to_writer(&mut *writer, message).expect("Json serialization failed");
                writeln!(writer).expect("Count not write newline to writer");
            }
            Self::JsonPretty => {
                serde_json::to_writer_pretty(&mut *writer, message).expect("JsonPretty serialization failed");
                writeln!(writer).expect("Count not write newline to writer");
            }
        }
    }
}
