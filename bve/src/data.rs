use include_dir::Dir;

pub const DATA: Dir<'static> = include_dir::include_dir!("data");
