use std::path::{Path, PathBuf};

#[must_use]
pub fn resolve_path(base: impl AsRef<Path>, path: PathBuf) -> Option<PathBuf> {
    let base = base.as_ref();
    let combined = base.join(path);
    if combined.exists() {
        return Some(combined.canonicalize().expect("Failed to canonicalize"));
    }

    let mut new_path = combined.clone();
    'comp: for component in combined.iter() {
        let path = new_path.join(component);
        if path.exists() {
            new_path = path;
        } else {
            let file_to_find = component.to_string_lossy().to_lowercase();
            for entry in new_path.read_dir().expect("Must be able to read dir") {
                let entry = entry.expect("DirEntry failed");
                let file_name = entry.file_name();
                let file_name_lower = file_name.to_string_lossy().to_lowercase();
                if file_name_lower == file_to_find {
                    new_path.push(entry.file_name());
                    continue 'comp;
                }
            }
            // We didn't find anything
            return None;
        }
    }

    Some(new_path.canonicalize().expect("Failed to canonicalize"))
}

#[cfg(test)]
mod test {
    use crate::filesystem::resolve_path;
    use std::path::PathBuf;

    #[test]
    fn filesystem_resolve_test() {
        let res = resolve_path(PathBuf::from("src"), PathBuf::from("fIlEsYsTeM/rEsOlVe.rs"));
        assert_eq!(
            res,
            Some(
                PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src/filesystem/resolve.rs"))
                    .canonicalize()
                    .expect("Failed to canonicalize")
            )
        );
    }
}
