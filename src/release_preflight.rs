use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Passed,
    Pending,
    Failed,
}

impl CheckStatus {
    fn marker(self) -> &'static str {
        match self {
            Self::Passed => "ok",
            Self::Pending => "pending",
            Self::Failed => "fail",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub name: &'static str,
    pub status: CheckStatus,
    pub detail: String,
}

impl CheckResult {
    fn pass(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Passed,
            detail: detail.into(),
        }
    }

    fn pending(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Pending,
            detail: detail.into(),
        }
    }

    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: CheckStatus::Failed,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightReport {
    pub version: String,
    pub checks: Vec<CheckResult>,
}

impl PreflightReport {
    pub fn ok(&self) -> bool {
        self.checks
            .iter()
            .all(|check| check.status != CheckStatus::Failed)
    }

    pub fn render(&self) -> String {
        let mut output = format!("release preflight for v{}\n", self.version);
        for check in &self.checks {
            output.push_str(&format!(
                "  [{}] {}: {}\n",
                check.status.marker(),
                check.name,
                check.detail
            ));
        }
        if self.ok() {
            output.push_str("\nrelease preflight checks passed");
            if self
                .checks
                .iter()
                .any(|check| check.status == CheckStatus::Pending)
            {
                output.push_str(" with pending live verification");
            }
            output.push_str(".\n");
        } else {
            output.push_str("\nrelease preflight checks failed.\n");
        }
        output
    }
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: CargoPackage,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
}

pub fn normalize_version(input: &str) -> String {
    let trimmed = input.trim();
    let without_ref = trimmed.strip_prefix("refs/tags/").unwrap_or(trimmed);
    let after_slash = without_ref.rsplit('/').next().unwrap_or(without_ref);
    let after_dash = after_slash.rsplit('-').next().unwrap_or(after_slash);
    after_dash
        .strip_prefix('v')
        .unwrap_or(after_dash)
        .to_string()
}

pub fn parse_cargo_toml(contents: &str) -> Result<(String, String), String> {
    let parsed: CargoToml =
        toml::from_str(contents).map_err(|error| format!("failed to parse Cargo.toml: {error}"))?;
    Ok((parsed.package.name, parsed.package.version))
}

pub fn check_cargo_toml(contents: &str, expected_version: &str) -> CheckResult {
    match parse_cargo_toml(contents) {
        Ok((name, version)) if version == expected_version => {
            CheckResult::pass("Cargo.toml version", format!("{name} = {version}"))
        }
        Ok((name, version)) => CheckResult::fail(
            "Cargo.toml version",
            format!("{name} is {version}, expected {expected_version}"),
        ),
        Err(error) => CheckResult::fail("Cargo.toml version", error),
    }
}

pub fn check_cargo_lock(contents: &str, package_name: &str, expected_version: &str) -> CheckResult {
    let name_needle = format!("name = \"{package_name}\"");
    let mut lines = contents.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() != name_needle {
            continue;
        }
        for lookahead in lines.by_ref().take(6) {
            let trimmed = lookahead.trim();
            if let Some(version) = trimmed
                .strip_prefix("version = \"")
                .and_then(|rest| rest.strip_suffix('"'))
            {
                return if version == expected_version {
                    CheckResult::pass(
                        "Cargo.lock freshness",
                        format!("{package_name} = {version}"),
                    )
                } else {
                    CheckResult::fail(
                        "Cargo.lock freshness",
                        format!("{package_name} is {version}, expected {expected_version}"),
                    )
                };
            }
        }
        return CheckResult::fail(
            "Cargo.lock freshness",
            format!("found {package_name} but no version line"),
        );
    }

    CheckResult::fail(
        "Cargo.lock freshness",
        format!("no [[package]] entry for {package_name}"),
    )
}

pub fn run_preflight(repo_root: &Path, raw_version: &str) -> Result<PreflightReport> {
    let expected_version = normalize_version(raw_version);
    let cargo_toml_path = repo_root.join("Cargo.toml");
    let cargo_lock_path = repo_root.join("Cargo.lock");
    let cargo_toml = read_required(&cargo_toml_path)?;
    let cargo_lock = read_required(&cargo_lock_path)?;
    let (package_name, _) = parse_cargo_toml(&cargo_toml)
        .map_err(|error| anyhow::anyhow!("{}: {error}", cargo_toml_path.display()))?;

    let checks = vec![
        check_cargo_toml(&cargo_toml, &expected_version),
        check_cargo_lock(&cargo_lock, &package_name, &expected_version),
        check_public_commands(repo_root),
        check_docs_commands(repo_root),
        check_hook_templates(repo_root),
        check_fixture_policy(repo_root),
        check_service_template(repo_root),
        check_live_verification(repo_root),
    ];

    Ok(PreflightReport {
        version: expected_version,
        checks,
    })
}

fn check_public_commands(repo_root: &Path) -> CheckResult {
    let path = repo_root.join("tests/fixtures/cli/public_commands.txt");
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) => {
            return CheckResult::fail(
                "public command fixture",
                format!("failed to read {}: {error}", path.display()),
            );
        }
    };
    let commands = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    let required = [
        "start",
        "status",
        "setup",
        "send ",
        "emit ",
        "explain ",
        "config show",
        "config path",
        "config verify",
        "hermes hook",
        "hermes install-hooks",
        "hermes uninstall-hooks",
        "git commit",
        "git branch-changed",
        "github issue-opened",
        "github pr-opened",
        "github check-failed",
        "github release-published",
        "install",
        "uninstall",
        "release preflight",
    ];
    let missing = required
        .into_iter()
        .filter(|prefix| !commands.iter().any(|command| command.starts_with(prefix)))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        CheckResult::pass(
            "public command fixture",
            "all required public commands are listed",
        )
    } else {
        CheckResult::fail(
            "public command fixture",
            format!("missing command prefixes: {}", missing.join(", ")),
        )
    }
}

fn check_docs_commands(repo_root: &Path) -> CheckResult {
    let readme = match fs::read_to_string(repo_root.join("README.md")) {
        Ok(raw) => raw,
        Err(error) => return CheckResult::fail("docs commands", error.to_string()),
    };
    let plan =
        fs::read_to_string(repo_root.join("docs/plans/2026-06-15-hermeship-development-plan.md"))
            .unwrap_or_default();
    let operations_path = repo_root.join("docs/operations.md");
    let operations = match fs::read_to_string(&operations_path) {
        Ok(raw) => raw,
        Err(error) => {
            return CheckResult::fail(
                "docs commands",
                format!("failed to read {}: {error}", operations_path.display()),
            );
        }
    };
    let combined = format!("{readme}\n{plan}\n{operations}");
    let required = [
        "hermeship setup",
        "hermeship install",
        "hermeship uninstall",
        "hermeship git commit",
        "hermeship git branch-changed",
        "hermeship github issue-opened",
        "hermeship github pr-opened",
        "hermeship github check-failed",
        "hermeship github release-published",
        "hermeship release preflight",
    ];
    let missing = required
        .into_iter()
        .filter(|needle| !combined.contains(needle))
        .collect::<Vec<_>>();
    let operations_required = [
        "--discord-token-stdin",
        "hermeship uninstall --remove-state --remove-config --remove-hooks",
    ];
    let operations_missing = operations_required
        .into_iter()
        .filter(|needle| !operations.contains(needle))
        .collect::<Vec<_>>();

    if missing.is_empty() && operations_missing.is_empty() {
        CheckResult::pass(
            "docs commands",
            "README/plan/operations mention lifecycle, git, and GitHub commands",
        )
    } else {
        let mut all_missing = missing;
        all_missing.extend(operations_missing);
        CheckResult::fail(
            "docs commands",
            format!("missing {}", all_missing.join(", ")),
        )
    }
}

fn check_hook_templates(repo_root: &Path) -> CheckResult {
    let manifest = fs::read_to_string(repo_root.join("templates/hermes-hook/HOOK.yaml"));
    let handler = fs::read_to_string(repo_root.join("templates/hermes-hook/handler.py"));
    match (manifest, handler) {
        (Ok(manifest), Ok(handler))
            if manifest.contains("name: hermeship")
                && manifest.contains("gateway:startup")
                && handler.contains("def handle(") =>
        {
            CheckResult::pass(
                "hook templates",
                "Hermes hook manifest and handler are bundled",
            )
        }
        (Ok(_), Ok(_)) => CheckResult::fail(
            "hook templates",
            "Hermes hook templates are present but missing required contract text",
        ),
        (manifest, handler) => CheckResult::fail(
            "hook templates",
            format!(
                "missing template file(s): manifest={}, handler={}",
                manifest
                    .err()
                    .map(|error| error.to_string())
                    .unwrap_or_default(),
                handler
                    .err()
                    .map(|error| error.to_string())
                    .unwrap_or_default()
            ),
        ),
    }
}

fn check_fixture_policy(repo_root: &Path) -> CheckResult {
    let path = repo_root.join("tests/fixtures/README.md");
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw.to_ascii_lowercase(),
        Err(error) => {
            return CheckResult::fail(
                "fixture policy",
                format!("failed to read {}: {error}", path.display()),
            );
        }
    };
    let required = [
        "synthetic",
        "tokens",
        "secrets",
        "full prompts",
        "full conversations",
    ];
    let missing = required
        .into_iter()
        .filter(|needle| !raw.contains(needle))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        CheckResult::pass("fixture policy", "fixture hygiene policy is explicit")
    } else {
        CheckResult::fail("fixture policy", format!("missing {}", missing.join(", ")))
    }
}

fn check_service_template(repo_root: &Path) -> CheckResult {
    let path = repo_root.join("deploy/hermeship.service");
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) => {
            return CheckResult::fail(
                "service template",
                format!("failed to read {}: {error}", path.display()),
            );
        }
    };
    let required = [
        "Environment=HERMESHIP_CONFIG=",
        "ExecStart=",
        "hermeship start",
    ];
    let missing = required
        .into_iter()
        .filter(|needle| !raw.contains(needle))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        CheckResult::pass(
            "service template",
            "systemd user service template is present",
        )
    } else {
        CheckResult::fail(
            "service template",
            format!("missing {}", missing.join(", ")),
        )
    }
}

fn check_live_verification(repo_root: &Path) -> CheckResult {
    let path = repo_root.join("docs/live-verification.md");
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(_) => {
            return CheckResult::pending(
                "live verification",
                "docs/live-verification.md not present; live verification remains manual",
            );
        }
    };
    let required = ["日期", "commit", "Discord", "Hermes", "回滚"];
    let missing = required
        .into_iter()
        .filter(|needle| !raw.contains(needle))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        CheckResult::pass(
            "live verification",
            "required live verification fields are present",
        )
    } else {
        CheckResult::pending(
            "live verification",
            format!("missing live verification fields: {}", missing.join(", ")),
        )
    }
}

fn read_required(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

    const CARGO_TOML_SAMPLE: &str = r#"
[package]
name = "hermeship"
version = "0.1.0"
edition = "2024"
"#;

    const CARGO_LOCK_SAMPLE: &str = r#"
version = 4

[[package]]
name = "anyhow"
version = "1.0.99"

[[package]]
name = "hermeship"
version = "0.1.0"
"#;

    #[test]
    fn normalize_version_accepts_common_tag_shapes() {
        assert_eq!(normalize_version("0.1.0"), "0.1.0");
        assert_eq!(normalize_version("v0.1.0"), "0.1.0");
        assert_eq!(normalize_version("refs/tags/v0.1.0"), "0.1.0");
        assert_eq!(normalize_version("hermeship-v0.1.0"), "0.1.0");
    }

    #[test]
    fn cargo_version_checks_fail_on_mismatch_or_stale_lock() {
        let cargo = check_cargo_toml(CARGO_TOML_SAMPLE, "0.1.1");
        assert_eq!(cargo.status, CheckStatus::Failed);
        assert!(cargo.detail.contains("0.1.0"));
        assert!(cargo.detail.contains("0.1.1"));

        let lock = check_cargo_lock(CARGO_LOCK_SAMPLE, "hermeship", "0.1.1");
        assert_eq!(lock.status, CheckStatus::Failed);
        assert!(lock.detail.contains("0.1.0"));
    }

    #[test]
    fn preflight_passes_local_checks_and_marks_missing_live_verification_pending() {
        let root = temp_dir("preflight-ok");
        write_project_fixture(&root, None);

        let report = run_preflight(&root, "v0.1.0").unwrap();

        assert!(report.ok(), "{}", report.render());
        assert!(report.checks.iter().any(|check| {
            check.name == "live verification" && check.status == CheckStatus::Pending
        }));
        assert!(report.render().contains("[pending] live verification"));

        remove_temp_dir(&root);
    }

    #[test]
    fn preflight_fails_when_public_command_fixture_omits_setup() {
        let root = temp_dir("preflight-command-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                public_commands: Some("start\ninstall\nuninstall\nrelease preflight 0.1.0\n"),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("setup"));

        remove_temp_dir(&root);
    }

    #[test]
    fn preflight_fails_when_public_command_fixture_omits_github_commands() {
        let root = temp_dir("preflight-github-command-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                public_commands: Some(
                    "start\nstatus\nsetup --default-channel ops\nsend --channel ops --message hello\nemit hermes.agent.started --payload '{}'\nexplain hermes.agent.started --payload '{}'\nconfig show\nconfig path\nconfig verify\nhermes hook --payload '{}'\nhermes install-hooks --scope global --force\nhermes uninstall-hooks --dry-run\ngit commit --repo hermeship --branch main --commit 1234567890abcdef1234567890abcdef12345678 --summary ship\ngit branch-changed --repo hermeship --old-branch main --new-branch codex/milestone-8-git\ninstall\nuninstall\nrelease preflight 0.1.0\n",
                ),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("github issue-opened"));
        assert!(report.render().contains("github pr-opened"));
        assert!(report.render().contains("github check-failed"));
        assert!(report.render().contains("github release-published"));

        remove_temp_dir(&root);
    }

    #[test]
    fn preflight_fails_when_docs_omit_github_pr_check_and_release_commands() {
        let root = temp_dir("preflight-github-docs-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                readme: Some(
                    "hermeship setup\nhermeship install\nhermeship uninstall\nhermeship git commit\nhermeship git branch-changed\nhermeship github issue-opened\nhermeship release preflight <version>\n",
                ),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("hermeship github pr-opened"));
        assert!(report.render().contains("hermeship github check-failed"));
        assert!(
            report
                .render()
                .contains("hermeship github release-published")
        );

        remove_temp_dir(&root);
    }

    #[test]
    fn preflight_fails_when_hook_template_is_missing() {
        let root = temp_dir("preflight-hook-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                hook_manifest: Some("name: other\n"),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("hook templates"));

        remove_temp_dir(&root);
    }

    #[test]
    fn preflight_fails_when_operations_doc_uses_plaintext_token_arg() {
        let root = temp_dir("preflight-operations-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                operations: Some(
                    "hermeship setup --discord-token <token>\nhermeship uninstall --remove-hooks\n",
                ),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("--discord-token-stdin"));

        remove_temp_dir(&root);
    }

    #[derive(Default)]
    struct ProjectFixtureOverrides {
        readme: Option<&'static str>,
        public_commands: Option<&'static str>,
        hook_manifest: Option<&'static str>,
        operations: Option<&'static str>,
    }

    fn write_project_fixture(root: &Path, overrides: Option<ProjectFixtureOverrides>) {
        let overrides = overrides.unwrap_or_default();
        write(root.join("Cargo.toml"), CARGO_TOML_SAMPLE);
        write(root.join("Cargo.lock"), CARGO_LOCK_SAMPLE);
        write(
            root.join("README.md"),
            overrides.readme.unwrap_or(
                "hermeship setup\nhermeship install\nhermeship uninstall\nhermeship git commit\nhermeship git branch-changed\nhermeship github issue-opened\nhermeship github pr-opened\nhermeship github check-failed\nhermeship github release-published\nhermeship release preflight <version>\n",
            ),
        );
        write(
            root.join("docs/operations.md"),
            overrides.operations.unwrap_or(
                "hermeship setup --discord-token-stdin --default-channel ops\nhermeship install\nhermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home ~/.hermes\nhermeship release preflight 0.1.0\n",
            ),
        );
        write(
            root.join("tests/fixtures/README.md"),
            "Fixtures must be synthetic and must not contain real tokens, cookies, secrets, full prompts, full conversations, or provider request/response bodies.\n",
        );
        write(
            root.join("tests/fixtures/cli/public_commands.txt"),
            overrides.public_commands.unwrap_or(
                "start\nstatus\nsetup --default-channel ops\nsend --channel ops --message hello\nemit hermes.agent.started --payload '{}'\nexplain hermes.agent.started --payload '{}'\nconfig show\nconfig path\nconfig verify\nhermes hook --payload '{}'\nhermes install-hooks --scope global --force\nhermes uninstall-hooks --dry-run\ngit commit --repo hermeship --branch main --commit 1234567890abcdef1234567890abcdef12345678 --summary ship\ngit branch-changed --repo hermeship --old-branch main --new-branch codex/milestone-8-git\ngithub issue-opened --owner posp --repo hermeship --number 42 --title issue\ngithub pr-opened --owner posp --repo hermeship --number 17 --title pr --branch codex/milestone-8-github\ngithub check-failed --owner posp --repo hermeship --workflow ci --status failure --branch main\ngithub release-published --owner posp --repo hermeship --tag v0.1.0\ninstall\nuninstall\nrelease preflight 0.1.0\n",
            ),
        );
        write(
            root.join("templates/hermes-hook/HOOK.yaml"),
            overrides
                .hook_manifest
                .unwrap_or("name: hermeship\nevents:\n  - gateway:startup\n"),
        );
        write(
            root.join("templates/hermes-hook/handler.py"),
            "def handle(event_type, context):\n    pass\n",
        );
        write(
            root.join("deploy/hermeship.service"),
            "[Service]\nEnvironment=HERMESHIP_CONFIG=%h/.hermeship/config.toml\nExecStart=%h/.cargo/bin/hermeship start\n",
        );
        fs::create_dir_all(root.join("docs")).unwrap();
    }

    fn write(path: PathBuf, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "hermeship-release-{name}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn remove_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
