// Temporary
pub trait UserError {
    fn print(&self) -> String
    where
        Self: ToString,
    {
        self.to_string()
    }
}

impl UserError for () {
    fn print(&self) -> String {
        unreachable!()
    }
}
