use crate::error::CskkError;
use std::env;
use std::env::VarError;
use std::path::PathBuf;

/// Get the data searching the data dir specified in freedesktop.
/// https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
///
/// filepath: relative path like "libcskk/foo/bar.toml"
// TODO: Not a scope of this class. Move to a different class?
// TODO: Replace caller with xdg::BaseDirectories
pub fn filepath_from_xdg_data_dir(filepath: &str) -> Result<String, CskkError> {
    let home_env = env::var("HOME");
    let xdg_data_home_env = env::var("XDG_DATA_HOME");
    let xdg_data_dirs_env = env::var("XDG_DATA_DIRS");

    search_path_from_xdg_data_dir(filepath, home_env, xdg_data_home_env, xdg_data_dirs_env)
}

fn search_path_from_xdg_data_dir(
    filepath: &str,
    home_env: Result<String, VarError>,
    xdg_data_home_env: Result<String, VarError>,
    xdg_data_dirs_env: Result<String, VarError>,
) -> Result<String, CskkError> {
    // XDG_DATA_HOME
    if let Ok(xdg_base_home) = xdg_data_home_env {
        if PathBuf::from(format!("{xdg_base_home}/{filepath}")).exists() {
            return Ok(format!("{xdg_base_home}/{filepath}"));
        }
    } else {
        // XDG_DATA_HOME default
        if let Ok(home) = home_env {
            if PathBuf::from(format!("{home}/.local/share/{filepath}")).exists() {
                return Ok(format!("{home}/.local/share/{filepath}"));
            }
        }
    }

    let xdg_data_dirs =
        xdg_data_dirs_env.unwrap_or_else(|_| "/usr/local/share/:/usr/share/".to_string());
    let xdg_data_dirs = xdg_data_dirs.split(':');
    for xdg_data_dir in xdg_data_dirs {
        if PathBuf::from(format!("{xdg_data_dir}/{filepath}")).exists() {
            return Ok(format!("{xdg_data_dir}/{filepath}"));
        }
    }

    Err(CskkError::Error("Not found".to_string()))
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    // Ignore until we can spy Pathbuf::exists()
    #[ignore]
    #[test]
    #[allow(unused_must_use)]
    fn search_xdg_dirs() {
        let home_env = Ok("/home/foo".to_string());
        let xdg_data_home_env = Err(VarError::NotPresent);
        let xdg_data_dirs_env = Err(VarError::NotPresent);

        search_path_from_xdg_data_dir("a/b", home_env, xdg_data_home_env, xdg_data_dirs_env);
    }
}
