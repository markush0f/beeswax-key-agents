use std::path::PathBuf;
use walkdir::WalkDir;

pub fn find_env_files(path: &str) -> Vec<PathBuf> {
    let mut env_files = Vec::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() {
            if let Some(filename) = p.file_name() {
                if let Some(name_str) = filename.to_str() {
                    // Buscar archivos que empiecen por .env, por ejemplo: .env, .env.local, .env.development
                    if name_str.starts_with(".env") {
                        env_files.push(p.to_path_buf());
                    }
                }
            }
        }
    }

    env_files
}
