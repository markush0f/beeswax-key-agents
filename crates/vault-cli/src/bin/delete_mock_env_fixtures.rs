use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use clap::Parser;

const DEFAULT_PREFIX: &str = "vault_env";

#[derive(Debug, Parser)]
#[command(
    name = "delete-mock-env-fixtures",
    about = "Delete generated mock .env fixture files matching a given prefix"
)]
struct Args {
    /// Root directory to scan for generated mock .env files.
    target_dir: PathBuf,

    /// Prefix used by the generator.
    #[arg(default_value = DEFAULT_PREFIX)]
    prefix: String,

    /// Print matching files without deleting them.
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    validate_args(&args)?;

    let mut matches = Vec::new();
    collect_matches(&args.target_dir, &args.prefix, &mut matches)?;
    matches.sort();

    for path in &matches {
        println!("{}", path.display());
        if !args.dry_run {
            fs::remove_file(path)?;
        }
    }

    let pattern = format!(".env.{}_*", args.prefix);
    if args.dry_run {
        println!(
            "dry-run: found {} files matching {} under {}",
            matches.len(),
            pattern,
            args.target_dir.display()
        );
    } else {
        println!(
            "deleted {} files matching {} under {}",
            matches.len(),
            pattern,
            args.target_dir.display()
        );
    }

    Ok(())
}

fn validate_args(args: &Args) -> Result<(), Box<dyn Error>> {
    if !args.target_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "target directory does not exist: {}",
                args.target_dir.display()
            ),
        )
        .into());
    }

    if !is_valid_prefix(&args.prefix) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "prefix must contain only letters, numbers, underscore or dash",
        )
        .into());
    }

    Ok(())
}

fn collect_matches(dir: &Path, prefix: &str, matches: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_matches(&path, prefix, matches)?;
            continue;
        }

        if is_generated_env_file(&path, prefix) {
            matches.push(path);
        }
    }

    Ok(())
}

fn is_generated_env_file(path: &Path, prefix: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with(&format!(".env.{prefix}_")))
        .unwrap_or(false)
}

fn is_valid_prefix(prefix: &str) -> bool {
    !prefix.is_empty()
        && prefix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{is_generated_env_file, is_valid_prefix};

    #[test]
    fn matches_generated_env_file_names() {
        assert!(is_generated_env_file(
            Path::new(".env.vault_env_001.local"),
            "vault_env"
        ));
        assert!(is_generated_env_file(
            Path::new(".env.vault_env_002.production"),
            "vault_env"
        ));
        assert!(!is_generated_env_file(Path::new(".env.local"), "vault_env"));
        assert!(!is_generated_env_file(
            Path::new(".env.other_prefix_001"),
            "vault_env"
        ));
    }

    #[test]
    fn validates_prefix_safely() {
        assert!(is_valid_prefix("vault_env"));
        assert!(is_valid_prefix("vault-env-2"));
        assert!(!is_valid_prefix("vault env"));
        assert!(!is_valid_prefix("../vault_env"));
    }
}
