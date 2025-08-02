use std::{env, fs, path::PathBuf};

use snafu::Snafu;

pub fn enumrate_executables() -> Result<impl Iterator<Item = PathBuf>> {
    struct Iter {
        path_dirs: std::vec::IntoIter<PathBuf>,
        current_dir_entries: Option<std::vec::IntoIter<PathBuf>>,
    }

    impl Iterator for Iter {
        type Item = PathBuf;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(ref mut entries) = self.current_dir_entries {
                    if let Some(entry) = entries.next() {
                        return Some(entry);
                    }
                    self.current_dir_entries = None;
                }

                let path_dir = self.path_dirs.next()?;

                if let Ok(entries) = fs::read_dir(&path_dir) {
                    let executable_files: Vec<PathBuf> = entries
                        .filter_map(|entry| {
                            let entry = entry.ok()?;
                            let path = entry.path();

                            if !path.is_file() {
                                return None;
                            }

                            let extension = path.extension()?.to_str()?;
                            if !matches!(extension.to_lowercase().as_str(), "exe" | "dll") {
                                return None;
                            }
                            Some(path)
                        })
                        .collect();

                    if !executable_files.is_empty() {
                        self.current_dir_entries = Some(executable_files.into_iter());
                    }
                }
            }
        }
    }

    let path_env = env::var("PATH").map_err(|_| Error::PathNotFound)?;
    let path_dirs: Vec<PathBuf> = path_env
        .split(';')
        .map(|dir| PathBuf::from(dir.trim()))
        .filter(|dir| dir.exists() && dir.is_dir())
        .collect();

    Ok(Iter {
        path_dirs: path_dirs.into_iter(),
        current_dir_entries: None,
    })
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("PATH environment variable not found"))]
    PathNotFound,

    #[snafu(display("IO error while reading directory: {}", source))]
    Io { source: std::io::Error },
}

type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumrate_executables() {
        let executables = enumrate_executables().unwrap();
        for exe_path in executables {
            println!("{exe_path:?}");
            assert!(
                exe_path.exists(),
                "Executable path does not exist: {exe_path:?}"
            );
        }
    }
}
