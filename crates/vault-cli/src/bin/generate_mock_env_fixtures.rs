use std::{
    error::Error,
    fmt::Write as _,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use clap::Parser;

const MOCK_FILE_PREFIX: &str = "vault_env";
const PROJECTS_PER_WORKSPACE: usize = 20;
const TOKEN_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
const ALPHANUMERIC_CHARSET: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const HEX_CHARSET: &[u8] = b"abcdef0123456789";

#[derive(Debug, Parser)]
#[command(
    name = "generate-mock-env-fixtures",
    about = "Generate nested .env fixtures with mock provider keys for scanner testing"
)]
struct Args {
    /// Root directory where nested fixture folders will be created.
    target_dir: PathBuf,

    /// Number of .env files to generate.
    #[arg(default_value_t = 50)]
    count: usize,

    /// Prefix embedded in the generated .env file name.
    #[arg(default_value = MOCK_FILE_PREFIX)]
    prefix: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    validate_args(&args)?;

    let created = generate_fixtures(&args.target_dir, args.count, &args.prefix)?;
    println!(
        "Created {created} mock .env files under {} using prefix {}",
        args.target_dir.display(),
        args.prefix
    );

    Ok(())
}

fn validate_args(args: &Args) -> Result<(), Box<dyn Error>> {
    if args.count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "count must be a positive integer",
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

fn generate_fixtures(target_dir: &Path, count: usize, prefix: &str) -> io::Result<usize> {
    fs::create_dir_all(target_dir)?;
    let mut random = RandomSource::new()?;

    for index in 1..=count {
        let workspace_id = (index - 1) / PROJECTS_PER_WORKSPACE + 1;
        let fixture_dir =
            target_dir.join(format!("workspace-{workspace_id:02}/project-{index:03}"));
        let fixture_file = fixture_dir.join(env_file_name(prefix, index));

        fs::create_dir_all(&fixture_dir)?;
        fs::write(
            &fixture_file,
            build_fixture_contents(index, prefix, &mut random)?,
        )?;
        println!("generated {}", fixture_file.display());
    }

    Ok(count)
}

fn build_fixture_contents(
    index: usize,
    prefix: &str,
    random: &mut RandomSource,
) -> io::Result<String> {
    let mut content = String::new();
    writeln!(
        content,
        "# Mock secrets generated for scanner testing only."
    )
    .unwrap();
    writeln!(content, "# Prefix marker: {prefix}").unwrap();
    writeln!(content, "APP_NAME=fixture-{index:03}").unwrap();
    writeln!(content, "NODE_ENV=test").unwrap();
    writeln!(content, "OPENAI_API_KEY={}", openai_key(index, random)?).unwrap();
    writeln!(content, "GPT_API_KEY={}", openai_key(index + 1, random)?).unwrap();
    writeln!(content, "OPENROUTER_API_KEY={}", openrouter_key(random)?).unwrap();
    writeln!(content, "GEMINI_API_KEY={}", gemini_key(random)?).unwrap();
    writeln!(content, "XAI_API_KEY={}", grok_key(random)?).unwrap();
    writeln!(content, "ANTHROPIC_API_KEY={}", anthropic_key(random)?).unwrap();
    writeln!(content, "OLLAMA_API_KEY={}", ollama_key(index, random)?).unwrap();
    writeln!(content, "DEEPSEEK_API_KEY={}", deepseek_key(random)?).unwrap();
    Ok(content)
}

fn env_file_name(prefix: &str, index: usize) -> String {
    let base = format!(".env.{prefix}_{index:03}");
    match index % 5 {
        0 => base,
        1 => format!("{base}.local"),
        2 => format!("{base}.development"),
        3 => format!("{base}.production"),
        _ => format!("{base}.test"),
    }
}

fn is_valid_prefix(prefix: &str) -> bool {
    !prefix.is_empty()
        && prefix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn openai_key(index: usize, random: &mut RandomSource) -> io::Result<String> {
    let token = random.sample(TOKEN_CHARSET, 40)?;
    if index % 2 == 0 {
        Ok(format!("sk-proj-{token}"))
    } else {
        Ok(format!("sk-{token}"))
    }
}

fn gemini_key(random: &mut RandomSource) -> io::Result<String> {
    Ok(format!("AIza{}", random.sample(TOKEN_CHARSET, 35)?))
}

fn openrouter_key(random: &mut RandomSource) -> io::Result<String> {
    Ok(format!("sk-or-v1-{}", random.sample(HEX_CHARSET, 64)?))
}

fn grok_key(random: &mut RandomSource) -> io::Result<String> {
    random.sample(TOKEN_CHARSET, 48)
}

fn anthropic_key(random: &mut RandomSource) -> io::Result<String> {
    Ok(format!("sk-ant-{}", random.sample(TOKEN_CHARSET, 32)?))
}

fn ollama_key(index: usize, random: &mut RandomSource) -> io::Result<String> {
    match index % 3 {
        0 => Ok(format!("ollama_{}", random.sample(TOKEN_CHARSET, 32)?)),
        1 => Ok(format!("sk-ollama-{}", random.sample(TOKEN_CHARSET, 32)?)),
        _ => Ok(format!(
            "{}.{}",
            random.sample(HEX_CHARSET, 32)?,
            random.sample(TOKEN_CHARSET, 24)?
        )),
    }
}

fn deepseek_key(random: &mut RandomSource) -> io::Result<String> {
    Ok(format!("sk-{}", random.sample(ALPHANUMERIC_CHARSET, 32)?))
}

struct RandomSource {
    file: File,
}

impl RandomSource {
    fn new() -> io::Result<Self> {
        Ok(Self {
            file: File::open("/dev/urandom")?,
        })
    }

    fn sample(&mut self, charset: &[u8], len: usize) -> io::Result<String> {
        let mut bytes = vec![0_u8; len];
        self.file.read_exact(&mut bytes)?;

        Ok(bytes
            .into_iter()
            .map(|byte| charset[usize::from(byte) % charset.len()] as char)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{env_file_name, is_valid_prefix};

    #[test]
    fn keeps_env_prefix_in_generated_file_names() {
        assert_eq!(env_file_name("vault_env", 1), ".env.vault_env_001.local");
        assert_eq!(env_file_name("vault_env", 5), ".env.vault_env_005");
    }

    #[test]
    fn accepts_only_safe_prefix_characters() {
        assert!(is_valid_prefix("vault_env"));
        assert!(is_valid_prefix("vault-env-2"));
        assert!(!is_valid_prefix("vault env"));
        assert!(!is_valid_prefix("../vault_env"));
    }
}
