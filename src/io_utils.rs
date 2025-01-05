use glob::{glob, PatternError};
use std::path::PathBuf;

pub(crate) fn glob_files_to_process(
    file_dir: &str,
    file_ext: &str,
) -> Result<Vec<PathBuf>, PatternError> {
    let mut file_paths: Vec<PathBuf> = Vec::new();
    match glob(&format!("{}/*.{}", file_dir, file_ext)) {
        Ok(globbed_file_paths) => {
            for fp in globbed_file_paths {
                match fp {
                    Ok(path_buf) => file_paths.push(path_buf),
                    Err(e) => println!("{:?}", e),
                }
            }
            Ok(file_paths)
        }
        Err(error) => Err(error),
    }
}
