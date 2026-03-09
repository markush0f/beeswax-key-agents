use std::{
    error::Error,
    fmt::Write as _,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use clap::Parser;

const MOCK_FILE_PREFIX: &str = "vault_file";
const PROJECTS_PER_WORKSPACE: usize = 20;
const TOKEN_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
const ALPHANUMERIC_CHARSET: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const HEX_CHARSET: &[u8] = b"abcdef0123456789";

#[derive(Debug, Parser)]
#[command(
    name = "generate-mock-file-fixtures",
    about = "Generate regular source/config files with hardcoded mock provider keys"
)]
struct Args {
    /// Root directory where nested fixture folders will be created.
    target_dir: PathBuf,

    /// Number of files to generate.
    #[arg(default_value_t = 50)]
    count: usize,

    /// Prefix embedded in the generated file name.
    #[arg(default_value = MOCK_FILE_PREFIX)]
    prefix: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    validate_args(&args)?;

    let created = generate_fixtures(&args.target_dir, args.count, &args.prefix)?;
    println!(
        "Created {created} mock source files under {} using prefix {}",
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
        let project_dir =
            target_dir.join(format!("workspace-{workspace_id:02}/project-{index:03}"));
        let relative_path = fixture_relative_path(prefix, index);
        let fixture_file = project_dir.join(relative_path);

        if let Some(parent) = fixture_file.parent() {
            fs::create_dir_all(parent)?;
        }

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
    let openai = openai_key(index, random)?;
    let openrouter = openrouter_key(random)?;
    let gemini = gemini_key(random)?;
    let grok = grok_key(random)?;
    let anthropic = anthropic_key(random)?;
    let ollama = ollama_key(index, random)?;
    let deepseek = deepseek_key(random)?;

    let marker = format!("{prefix}_{index:03}");
    let extension = file_extension(index);

    Ok(match extension {
        "rs" => rust_fixture(
            &marker,
            &openai,
            &openrouter,
            &gemini,
            &grok,
            &anthropic,
            &ollama,
            &deepseek,
        ),
        "ts" => ts_fixture(
            &marker,
            &openai,
            &openrouter,
            &gemini,
            &grok,
            &anthropic,
            &ollama,
            &deepseek,
        ),
        "py" => py_fixture(
            &marker,
            &openai,
            &openrouter,
            &gemini,
            &grok,
            &anthropic,
            &ollama,
            &deepseek,
        ),
        "json" => json_fixture(
            &marker,
            &openai,
            &openrouter,
            &gemini,
            &grok,
            &anthropic,
            &ollama,
            &deepseek,
        ),
        "yaml" => yaml_fixture(
            &marker,
            &openai,
            &openrouter,
            &gemini,
            &grok,
            &anthropic,
            &ollama,
            &deepseek,
        ),
        _ => toml_fixture(
            &marker,
            &openai,
            &openrouter,
            &gemini,
            &grok,
            &anthropic,
            &ollama,
            &deepseek,
        ),
    })
}

fn fixture_relative_path(prefix: &str, index: usize) -> PathBuf {
    let file_name = format!("{prefix}_{index:03}.{}", file_extension(index));
    match index % 6 {
        0 => PathBuf::from(format!("src/{file_name}")),
        1 => PathBuf::from(format!("app/{file_name}")),
        2 => PathBuf::from(format!("config/{file_name}")),
        3 => PathBuf::from(format!("services/{file_name}")),
        4 => PathBuf::from(format!("scripts/{file_name}")),
        _ => PathBuf::from(file_name),
    }
}

fn file_extension(index: usize) -> &'static str {
    match index % 6 {
        0 => "rs",
        1 => "ts",
        2 => "py",
        3 => "json",
        4 => "yaml",
        _ => "toml",
    }
}

fn rust_fixture(
    marker: &str,
    openai: &str,
    openrouter: &str,
    gemini: &str,
    grok: &str,
    anthropic: &str,
    ollama: &str,
    deepseek: &str,
) -> String {
    format!(
        "// Mock fixture marker: {marker}\n\
         pub const OPENAI_API_KEY: &str = \"{openai}\";\n\
         pub const OPENROUTER_API_KEY: &str = \"{openrouter}\";\n\
         pub const GEMINI_API_KEY: &str = \"{gemini}\";\n\
         pub const XAI_API_KEY: &str = \"{grok}\";\n\
         pub const ANTHROPIC_API_KEY: &str = \"{anthropic}\";\n\
         pub const OLLAMA_API_KEY: &str = \"{ollama}\";\n\
         pub const DEEPSEEK_API_KEY: &str = \"{deepseek}\";\n"
    )
}

fn ts_fixture(
    marker: &str,
    openai: &str,
    openrouter: &str,
    gemini: &str,
    grok: &str,
    anthropic: &str,
    ollama: &str,
    deepseek: &str,
) -> String {
    format!(
        "// Mock fixture marker: {marker}\n\
         export const openAiApiKey = \"{openai}\";\n\
         export const openRouterApiKey = \"{openrouter}\";\n\
         export const geminiApiKey = \"{gemini}\";\n\
         export const xaiApiKey = \"{grok}\";\n\
         export const anthropicApiKey = \"{anthropic}\";\n\
         export const ollamaApiKey = \"{ollama}\";\n\
         export const deepseekApiKey = \"{deepseek}\";\n"
    )
}

fn py_fixture(
    marker: &str,
    openai: &str,
    openrouter: &str,
    gemini: &str,
    grok: &str,
    anthropic: &str,
    ollama: &str,
    deepseek: &str,
) -> String {
    format!(
        "# Mock fixture marker: {marker}\n\
         OPENAI_API_KEY = \"{openai}\"\n\
         OPENROUTER_API_KEY = \"{openrouter}\"\n\
         GEMINI_API_KEY = \"{gemini}\"\n\
         XAI_API_KEY = \"{grok}\"\n\
         ANTHROPIC_API_KEY = \"{anthropic}\"\n\
         OLLAMA_API_KEY = \"{ollama}\"\n\
         DEEPSEEK_API_KEY = \"{deepseek}\"\n"
    )
}

fn json_fixture(
    marker: &str,
    openai: &str,
    openrouter: &str,
    gemini: &str,
    grok: &str,
    anthropic: &str,
    ollama: &str,
    deepseek: &str,
) -> String {
    format!(
        "{{\n  \"marker\": \"{marker}\",\n  \"openaiApiKey\": \"{openai}\",\n  \"openrouterApiKey\": \"{openrouter}\",\n  \"geminiApiKey\": \"{gemini}\",\n  \"xaiApiKey\": \"{grok}\",\n  \"anthropicApiKey\": \"{anthropic}\",\n  \"ollamaApiKey\": \"{ollama}\",\n  \"deepseekApiKey\": \"{deepseek}\"\n}}\n"
    )
}

fn yaml_fixture(
    marker: &str,
    openai: &str,
    openrouter: &str,
    gemini: &str,
    grok: &str,
    anthropic: &str,
    ollama: &str,
    deepseek: &str,
) -> String {
    format!(
        "marker: {marker}\nopenai_api_key: {openai}\nopenrouter_api_key: {openrouter}\ngemini_api_key: {gemini}\nxai_api_key: {grok}\nanthropic_api_key: {anthropic}\nollama_api_key: {ollama}\ndeepseek_api_key: {deepseek}\n"
    )
}

fn toml_fixture(
    marker: &str,
    openai: &str,
    openrouter: &str,
    gemini: &str,
    grok: &str,
    anthropic: &str,
    ollama: &str,
    deepseek: &str,
) -> String {
    let mut content = String::new();
    writeln!(content, "marker = \"{marker}\"").unwrap();
    writeln!(content, "openai_api_key = \"{openai}\"").unwrap();
    writeln!(content, "openrouter_api_key = \"{openrouter}\"").unwrap();
    writeln!(content, "gemini_api_key = \"{gemini}\"").unwrap();
    writeln!(content, "xai_api_key = \"{grok}\"").unwrap();
    writeln!(content, "anthropic_api_key = \"{anthropic}\"").unwrap();
    writeln!(content, "ollama_api_key = \"{ollama}\"").unwrap();
    writeln!(content, "deepseek_api_key = \"{deepseek}\"").unwrap();
    content
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
