//! Compile-time configuration constants for directory filtering and IDE root discovery.
//!
//! These slices are used by the file system walking and filtering logic in
//! [`crate::file_utils`] and [`crate::scan`] to skip irrelevant or unsafe directories
//! during recursive scans.

/// Directory names excluded from general project and environment file scans.
///
/// This list covers a broad range of common tool artifacts, build outputs, package
/// manager stores, and hidden configuration directories that are unlikely to contain
/// user-authored secrets and would significantly inflate scan time if included.
///
/// # Exclusion Scope
///
/// These exclusions apply to [`crate::scan::scan_all_files_for_keys`] and
/// [`crate::scan::scan_project_files_for_keys_streaming`]. The IDE scanner
/// ([`crate::scan::scan_ide_files_for_keys_streaming`]) operates on an explicit allowlist
/// instead and is **not** restricted by this list.
///
/// # Categories
///
/// - **VCS**: `.git`, `.hg`, `.svn`, `.bzr`, `.repo`
/// - **CI/CD**: `.github`, `.gitlab`, `.circleci`, `.azure-pipelines`
/// - **IDEs**: `.idea`, `.vscode`, `.vs`, `.fleet`
/// - **Build outputs**: `target`, `dist`, `build`, `out`, `coverage`
/// - **Package managers**: `node_modules`, `.yarn`, `.pnpm-store`, `.npm`, `.m2`, `.gradle`
/// - **Virtual envs**: `.venv`, `venv`, `env`
/// - **Framework caches**: `.next`, `.nuxt`, `.turbo`, `.angular`, `.expo`
/// - **IaC tools**: `.terraform`, `.pulumi`, `.serverless`, `.sst`
/// - **Misc caches**: `.vault-cache`, `.cache`, `.pytest_cache`, `.mypy_cache`, `tmp`, `temp`
pub const EXCLUDED_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".bzr",
    ".repo",
    ".github",
    ".gitlab",
    ".circleci",
    ".azure-pipelines",
    ".idea",
    ".vscode",
    ".vs",
    ".fleet",
    ".antigravity-server",
    ".codex",
    ".vault-cache",
    ".cache",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".nox",
    ".venv",
    "venv",
    "env",
    ".gradle",
    ".m2",
    ".ivy2",
    ".sbt",
    ".bsp",
    ".lein",
    ".terraform",
    ".terragrunt-cache",
    ".pulumi",
    ".serverless",
    ".sst",
    ".next",
    ".nuxt",
    ".parcel-cache",
    ".turbo",
    ".storybook",
    ".vite",
    ".expo",
    ".angular",
    ".nx",
    ".yarn",
    ".pnpm-store",
    ".npm",
    "target",
    "node_modules",
    "dist",
    "build",
    "out",
    "coverage",
    "tmp",
    "temp",
    "logs",
    "log",
    ".vscode-server",
];

/// Directory names targeted explicitly by the IDE-specific scanner.
///
/// Unlike [`EXCLUDED_DIRS`], which is a denylist, this list is an explicit allowlist
/// of IDE configuration roots relative to the project root. These directories often
/// contain settings files or cached AI completions that may inadvertently embed secrets.
///
/// Used by [`crate::scan::scan_ide_files_for_keys_streaming`].
pub const IDE_DIRS: &[&str] = &[".antigravity-server", ".vscode", ".idea"];
