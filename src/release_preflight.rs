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
        check_observer_plugin_template(repo_root),
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
        "hermes install-plugin",
        "hermes enable-plugin",
        "git commit",
        "git branch-changed",
        "github issue-opened",
        "github pr-opened",
        "github check-failed",
        "github release-published",
        "tmux keyword",
        "tmux stale",
        "tmux watch",
        "tmux list",
        "cron run",
        "memory init",
        "memory status",
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
        "hermeship tmux keyword",
        "hermeship tmux stale",
        "hermeship tmux watch",
        "hermeship tmux list",
        "hermeship cron run",
        "hermeship memory init",
        "hermeship memory status",
        "hermeship release preflight",
        "hermeship hermes install-plugin",
        "hermeship hermes enable-plugin",
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
            "README/plan/operations mention lifecycle, git, GitHub, tmux, cron, and memory commands",
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

fn check_observer_plugin_template(repo_root: &Path) -> CheckResult {
    let manifest_path = repo_root.join("templates/hermes-plugin/plugin.yaml");
    let init_path = repo_root.join("templates/hermes-plugin/__init__.py");
    let manifest = fs::read_to_string(&manifest_path);
    let init = fs::read_to_string(&init_path);
    let (manifest, init) = match (manifest, init) {
        (Ok(manifest), Ok(init)) => (manifest, init),
        (manifest, init) => {
            return CheckResult::fail(
                "observer plugin template",
                format!(
                    "missing template file(s): plugin.yaml={}, __init__.py={}",
                    manifest
                        .err()
                        .map(|error| error.to_string())
                        .unwrap_or_default(),
                    init.err()
                        .map(|error| error.to_string())
                        .unwrap_or_default()
                ),
            );
        }
    };
    let manifest_required = ["name: hermeship-observer"];
    let init_required = [
        "def register(ctx):",
        "ctx.register_hook",
        "on_session_start",
        "on_session_end",
        "on_session_finalize",
        "on_session_reset",
        "pre_api_request",
        "post_api_request",
        "api_request_error",
        "pre_llm_call",
        "post_llm_call",
        "pre_tool_call",
        "post_tool_call",
        "pre_approval_request",
        "post_approval_response",
        "subagent_start",
        "subagent_stop",
        "hermes.observer.tool.started",
        "hermes.observer.tool.finished",
        "hermes.observer.api.request.failed",
        "HERMESHIP_DAEMON_URL",
        "HERMESHIP_OBSERVER_TIMEOUT_SECS",
        "HERMESHIP_OBSERVER_DISABLED",
        "/event",
        "provider",
        "source",
        "observer_schema_version",
        "urllib.request",
        "return None",
    ];
    let mut missing = manifest_required
        .into_iter()
        .filter(|needle| !manifest.contains(needle))
        .map(|needle| format!("plugin.yaml:{needle}"))
        .collect::<Vec<_>>();
    missing.extend(
        init_required
            .into_iter()
            .filter(|needle| !init.contains(needle))
            .map(|needle| format!("__init__.py:{needle}")),
    );
    let forbidden = [
        "transform_tool_result",
        "\"action\": \"block\"",
        "\"action\":\"block\"",
        "'action': 'block'",
        "'action':'block'",
        "register_middleware",
        "ctx.register_middleware",
        "/api/hermes/hook",
        "DISCORD_TOKEN",
        "HERMESHIP_DISCORD_TOKEN",
    ];
    let present_forbidden = forbidden
        .into_iter()
        .filter(|needle| init.contains(needle) || manifest.contains(needle))
        .collect::<Vec<_>>();
    if missing.is_empty() && present_forbidden.is_empty() {
        CheckResult::pass(
            "observer plugin template",
            "Hermes observer plugin template is bundled",
        )
    } else {
        let mut details = Vec::new();
        if !missing.is_empty() {
            details.push(format!("missing {}", missing.join(", ")));
        }
        if !present_forbidden.is_empty() {
            details.push(format!("forbidden {}", present_forbidden.join(", ")));
        }
        CheckResult::fail("observer plugin template", details.join("; "))
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
            "live verification record fields are present; real live pass is not asserted",
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
    use std::process::Command;

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
    const OBSERVER_PLUGIN_MANIFEST_TEMPLATE: &str =
        include_str!("../templates/hermes-plugin/plugin.yaml");
    const OBSERVER_PLUGIN_INIT_TEMPLATE: &str =
        include_str!("../templates/hermes-plugin/__init__.py");

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
        assert!(
            !report
                .render()
                .contains("[ok] live verification: required live verification fields are present")
        );

        remove_temp_dir(&root);
    }

    #[test]
    fn preflight_live_verification_ok_says_record_fields_only() {
        let root = temp_dir("preflight-live-record-fields");
        write_project_fixture(&root, None);
        write(
            root.join("docs/live-verification.md"),
            "# Live Verification\n\n日期: 2026-06-19\ncommit: synthetic\nDiscord: not_run\nHermes: not_run\n回滚: not_run\n",
        );

        let report = run_preflight(&root, "v0.1.0").unwrap();
        let rendered = report.render();

        assert!(report.ok(), "{rendered}");
        assert!(rendered.contains(
            "[ok] live verification: live verification record fields are present; real live pass is not asserted"
        ));
        assert!(
            !rendered
                .contains("[ok] live verification: required live verification fields are present")
        );

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
    fn preflight_fails_when_public_command_fixture_omits_tmux_commands() {
        let root = temp_dir("preflight-tmux-command-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                public_commands: Some(
                    "start\nstatus\nsetup --default-channel ops\nsend --channel ops --message hello\nemit hermes.agent.started --payload '{}'\nexplain hermes.agent.started --payload '{}'\nconfig show\nconfig path\nconfig verify\nhermes hook --payload '{}'\nhermes install-hooks --scope global --force\nhermes uninstall-hooks --dry-run\ngit commit --repo hermeship --branch main --commit 1234567890abcdef1234567890abcdef12345678 --summary ship\ngit branch-changed --repo hermeship --old-branch main --new-branch codex/milestone-8-git\ngithub issue-opened --owner posp --repo hermeship --number 42 --title issue\ngithub pr-opened --owner posp --repo hermeship --number 17 --title pr --branch codex/milestone-8-github\ngithub check-failed --owner posp --repo hermeship --workflow ci --status failure --branch main\ngithub release-published --owner posp --repo hermeship --tag v0.1.0\ninstall\nuninstall\nrelease preflight 0.1.0\n",
                ),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("tmux keyword"));
        assert!(report.render().contains("tmux stale"));
        assert!(report.render().contains("tmux watch"));
        assert!(report.render().contains("tmux list"));

        remove_temp_dir(&root);
    }

    #[test]
    fn preflight_fails_when_public_command_fixture_omits_cron_or_memory_commands() {
        let root = temp_dir("preflight-cron-memory-command-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                public_commands: Some(
                    "start\nstatus\nsetup --default-channel ops\nsend --channel ops --message hello\nemit hermes.agent.started --payload '{}'\nexplain hermes.agent.started --payload '{}'\nconfig show\nconfig path\nconfig verify\nhermes hook --payload '{}'\nhermes install-hooks --scope global --force\nhermes uninstall-hooks --dry-run\ngit commit --repo hermeship --branch main --commit 1234567890abcdef1234567890abcdef12345678 --summary ship\ngit branch-changed --repo hermeship --old-branch main --new-branch codex/milestone-8-git\ngithub issue-opened --owner posp --repo hermeship --number 42 --title issue\ngithub pr-opened --owner posp --repo hermeship --number 17 --title pr --branch codex/milestone-8-github\ngithub check-failed --owner posp --repo hermeship --workflow ci --status failure --branch main\ngithub release-published --owner posp --repo hermeship --tag v0.1.0\ntmux keyword --session hermes-agent --keyword FAILED --line failed\ntmux stale --session hermes-agent --pane %2 --minutes 15 --last-line waiting\ntmux watch --session hermes-agent --keywords FAILED,complete --stale-minutes 10 --tmux-output 'hermes-agent\tmain\t%1\t0\tbash\tready'\ntmux list --tmux-output 'hermes-agent\tmain\t%1\t0\tbash\tready'\ninstall\nuninstall\nrelease preflight 0.1.0\n",
                ),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("cron run"));
        assert!(report.render().contains("memory init"));
        assert!(report.render().contains("memory status"));

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
    fn preflight_fails_when_docs_omit_tmux_commands() {
        let root = temp_dir("preflight-tmux-docs-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                readme: Some(
                    "hermeship setup\nhermeship install\nhermeship uninstall\nhermeship git commit\nhermeship git branch-changed\nhermeship github issue-opened\nhermeship github pr-opened\nhermeship github check-failed\nhermeship github release-published\nhermeship release preflight <version>\n",
                ),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("hermeship tmux keyword"));
        assert!(report.render().contains("hermeship tmux stale"));
        assert!(report.render().contains("hermeship tmux watch"));
        assert!(report.render().contains("hermeship tmux list"));

        remove_temp_dir(&root);
    }

    #[test]
    fn preflight_fails_when_docs_omit_cron_or_memory_commands() {
        let root = temp_dir("preflight-cron-memory-docs-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                readme: Some(
                    "hermeship setup\nhermeship install\nhermeship uninstall\nhermeship git commit\nhermeship git branch-changed\nhermeship github issue-opened\nhermeship github pr-opened\nhermeship github check-failed\nhermeship github release-published\nhermeship tmux keyword\nhermeship tmux stale\nhermeship tmux watch\nhermeship tmux list\nhermeship release preflight <version>\n",
                ),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("hermeship cron run"));
        assert!(report.render().contains("hermeship memory init"));
        assert!(report.render().contains("hermeship memory status"));

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
    fn preflight_fails_when_observer_plugin_template_is_missing() {
        let root = temp_dir("preflight-observer-plugin-fail");
        write_project_fixture(
            &root,
            Some(ProjectFixtureOverrides {
                observer_plugin_manifest: Some("name: other\n"),
                observer_plugin_init: Some("def register(ctx):\n    pass\n"),
                ..ProjectFixtureOverrides::default()
            }),
        );

        let report = run_preflight(&root, "0.1.0").unwrap();

        assert!(!report.ok());
        assert!(report.render().contains("observer plugin template"));

        remove_temp_dir(&root);
    }

    #[test]
    fn observer_plugin_template_compiles_with_python() {
        let output = Command::new("python3")
            .arg("-m")
            .arg("py_compile")
            .arg(observer_plugin_init_path())
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "observer plugin did not compile: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn observer_plugin_smoke_registers_hooks_and_forwards_safe_fields() {
        let script = format!(
            r#"
import importlib.util
import json
import os

spec = importlib.util.spec_from_file_location("hermeship_observer_test", {plugin_path:?})
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

events = []

class FakeResponse:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, traceback):
        return False

    def read(self):
        return b"ok"

def fake_urlopen(request, timeout=None):
    events.append({{
        "url": request.full_url,
        "method": request.get_method(),
        "content_type": request.get_header("Content-type"),
        "timeout": timeout,
        "body": request.data.decode("utf-8"),
    }})
    return FakeResponse()

module.urllib.request.urlopen = fake_urlopen
os.environ["HERMESHIP_DAEMON_URL"] = "http://127.0.0.1:25295/"
os.environ["HERMESHIP_OBSERVER_TIMEOUT_SECS"] = "999999"
os.environ.pop("HERMESHIP_OBSERVER_DISABLED", None)

class FakeContext:
    def __init__(self):
        self.hooks = {{}}

    def register_hook(self, name, callback):
        self.hooks[name] = callback

class ApprovalContext:
    session_key = "object-secret-session-key-do-not-forward"
    surface = "terminal"
    pattern_key = "object-pattern"
    turn_id = "object-turn"
    tool_call_id = "object-tool"
    description = "RAW_OBJECT_APPROVAL_DESCRIPTION_DO_NOT_FORWARD"
    command = "RAW_OBJECT_APPROVAL_COMMAND_DO_NOT_FORWARD"

ctx = FakeContext()
assert module.register(ctx) is None
expected_hooks = {{
    "on_session_start",
    "on_session_end",
    "on_session_finalize",
    "on_session_reset",
    "pre_api_request",
    "post_api_request",
    "api_request_error",
    "pre_llm_call",
    "post_llm_call",
    "pre_tool_call",
    "post_tool_call",
    "pre_approval_request",
    "post_approval_response",
    "subagent_start",
    "subagent_stop",
}}
assert set(ctx.hooks) == expected_hooks, sorted(set(ctx.hooks) ^ expected_hooks)
assert "transform_tool_result" not in ctx.hooks

assert ctx.hooks["pre_tool_call"]({{
    "session_id": "session-1",
    "task_id": "task-1",
    "turn_id": "turn-1",
    "api_request_id": "api-1",
    "tool_call_id": "tool-1",
    "tool_name": "terminal",
    "arguments": {{"path": "/tmp/demo", "mode": "read"}},
    "command": "RAW_COMMAND_DO_NOT_FORWARD",
    "tool_result": "RAW_TOOL_RESULT_DO_NOT_FORWARD",
    "request": {{"body": "RAW_REQUEST_DO_NOT_FORWARD"}},
}}) is None
assert ctx.hooks["post_tool_call"]({{
    "session_id": "session-1",
    "tool_call_id": "tool-1",
    "tool_name": "terminal",
    "status": "ok",
    "duration_ms": 42,
    "result": "RAW_RESULT_DO_NOT_FORWARD",
}}) is None
assert ctx.hooks["post_llm_call"]({{
    "session_id": "session-1",
    "platform": "telegram",
    "model": "synthetic-model",
    "response": "RAW_RESPONSE_DO_NOT_FORWARD",
}}) is None
assert ctx.hooks["api_request_error"]({{
    "session_id": "session-1",
    "task_id": "task-1",
    "turn_id": "turn-1",
    "api_request_id": "api-1",
    "provider": "synthetic-provider",
    "model": "synthetic-model",
    "api_mode": "chat",
    "api_call_count": 3,
    "error_type": "RuntimeError: RAW_ERROR_TYPE_DO_NOT_FORWARD",
    "error_message": "RAW_ERROR_MESSAGE_DO_NOT_FORWARD",
    "error_summary": "bounded error summary",
    "error": RuntimeError("RAW_ERROR_BODY_DO_NOT_FORWARD"),
    "duration_ms": 7,
}}) is None
assert ctx.hooks["post_tool_call"]({{
    "session_id": "session-1",
    "tool_call_id": "tool-2",
    "tool_name": "terminal",
    "status": "failed with RAW_STATUS_TEXT_DO_NOT_FORWARD",
    "error_message": "RAW_TOOL_ERROR_MESSAGE_DO_NOT_FORWARD",
    "error_summary": "tool failed safely",
    "result": "RAW_FAILED_TOOL_RESULT_DO_NOT_FORWARD",
}}) is None
assert ctx.hooks["subagent_start"]({{
    "parent_session_id": "parent-session",
    "parent_turn_id": "parent-turn",
    "parent_subagent_id": "parent-subagent",
    "child_session_id": "child-session",
    "child_subagent_id": "child-subagent",
    "child_role": "reviewer",
    "child_goal": "RAW_CHILD_GOAL_DO_NOT_FORWARD",
}}) is None
assert ctx.hooks["pre_approval_request"]({{
    "session_key": "sk-secret-session-key-do-not-forward",
    "surface": "terminal",
    "pattern_key": "shell-command",
    "pattern_keys": [
        "pattern-" + str(index) + "-RAW_PATTERN_KEY_SHOULD_BE_BOUNDED_" + ("x" * 80)
        for index in range(20)
    ],
    "description": "RAW_APPROVAL_DESCRIPTION_DO_NOT_FORWARD",
    "command": "RAW_APPROVAL_COMMAND_DO_NOT_FORWARD",
    "turn_id": "turn-approval",
    "tool_call_id": "tool-approval",
}}) is None
assert ctx.hooks["pre_approval_request"](ApprovalContext()) is None
assert ctx.hooks["subagent_stop"]({{
    "parent_session_id": "parent-session",
    "parent_turn_id": "parent-turn",
    "child_session_id": "child-session",
    "child_role": "reviewer",
    "child_status": "done with RAW_CHILD_STATUS_DO_NOT_FORWARD",
    "child_summary": "RAW_CHILD_SUMMARY_DO_NOT_FORWARD",
    "duration_ms": 9,
}}) is None

payloads = [json.loads(event["body"]) for event in events]
assert any(payload["type"] == "hermes.observer.tool.started" for payload in payloads)
assert any(payload["type"] == "hermes.observer.tool.finished" for payload in payloads)
assert any(payload["type"] == "hermes.observer.llm.finished" for payload in payloads)
assert any(payload["type"] == "hermes.observer.api.request.failed" for payload in payloads)
assert any(payload["type"] == "hermes.observer.approval.requested" for payload in payloads)
assert any(payload["type"] == "hermes.observer.subagent.started" for payload in payloads)
assert any(payload["type"] == "hermes.observer.subagent.finished" for payload in payloads)

for event in events:
    assert event["url"] == "http://127.0.0.1:25295/event"
    assert "/api/hermes/hook" not in event["url"]
    assert event["method"] == "POST"
    assert event["content_type"] == "application/json"
    assert event["timeout"] == 5.0

encoded = json.dumps(payloads, sort_keys=True)
for forbidden in [
    "RAW_COMMAND_DO_NOT_FORWARD",
    "RAW_TOOL_RESULT_DO_NOT_FORWARD",
    "RAW_REQUEST_DO_NOT_FORWARD",
    "RAW_RESULT_DO_NOT_FORWARD",
    "RAW_RESPONSE_DO_NOT_FORWARD",
    "RAW_ERROR_MESSAGE_DO_NOT_FORWARD",
    "RAW_ERROR_BODY_DO_NOT_FORWARD",
    "RAW_ERROR_TYPE_DO_NOT_FORWARD",
    "RAW_TOOL_ERROR_MESSAGE_DO_NOT_FORWARD",
    "RAW_FAILED_TOOL_RESULT_DO_NOT_FORWARD",
    "RAW_CHILD_GOAL_DO_NOT_FORWARD",
    "RAW_APPROVAL_DESCRIPTION_DO_NOT_FORWARD",
    "RAW_APPROVAL_COMMAND_DO_NOT_FORWARD",
    "RAW_OBJECT_APPROVAL_DESCRIPTION_DO_NOT_FORWARD",
    "RAW_OBJECT_APPROVAL_COMMAND_DO_NOT_FORWARD",
    "RAW_CHILD_SUMMARY_DO_NOT_FORWARD",
    "RAW_STATUS_TEXT_DO_NOT_FORWARD",
    "RAW_CHILD_STATUS_DO_NOT_FORWARD",
    "sk-secret-session-key-do-not-forward",
    "object-secret-session-key-do-not-forward",
]:
    assert forbidden not in encoded, forbidden

tool_started = next(payload for payload in payloads if payload["type"] == "hermes.observer.tool.started")
assert tool_started["payload"]["provider"] == "hermes"
assert tool_started["payload"]["source"] == "plugin"
assert tool_started["payload"]["observer_schema_version"] == 1
assert tool_started["payload"]["arg_keys"] == ["mode", "path"]
assert tool_started["payload"]["arg_key_count"] == 2
assert isinstance(tool_started["payload"]["arg_chars"], int)
api_failed = next(payload for payload in payloads if payload["type"] == "hermes.observer.api.request.failed")
assert api_failed["payload"]["error_message"] == "bounded error summary"
assert api_failed["payload"]["error_type"] == "RuntimeError"
assert api_failed["payload"]["error_type_chars"] == len("RuntimeError: RAW_ERROR_TYPE_DO_NOT_FORWARD")
tool_failed = [
    payload for payload in payloads
    if payload["type"] == "hermes.observer.tool.finished"
    and payload["payload"].get("tool_call_id") == "tool-2"
][0]
assert tool_failed["payload"]["error_message"] == "tool failed safely"
assert "status" not in tool_failed["payload"]
assert tool_failed["payload"]["status_chars"] == len("failed with RAW_STATUS_TEXT_DO_NOT_FORWARD")
approval_requested = next(payload for payload in payloads if payload["type"] == "hermes.observer.approval.requested")
assert "session_key" not in approval_requested["payload"]
assert approval_requested["payload"]["session_key_chars"] == len("sk-secret-session-key-do-not-forward")
assert approval_requested["payload"]["has_session_key"] is True
assert approval_requested["payload"]["description_chars"] == len("RAW_APPROVAL_DESCRIPTION_DO_NOT_FORWARD")
assert approval_requested["payload"]["command_chars"] == len("RAW_APPROVAL_COMMAND_DO_NOT_FORWARD")
assert approval_requested["payload"]["pattern_key_count"] == 20
assert len(approval_requested["payload"]["pattern_keys"]) == 16
assert all(len(value) <= 64 for value in approval_requested["payload"]["pattern_keys"])
object_approval_requested = [
    payload for payload in payloads
    if payload["type"] == "hermes.observer.approval.requested"
    and payload["payload"].get("turn_id") == "object-turn"
][0]
assert "session_key" not in object_approval_requested["payload"]
assert object_approval_requested["payload"]["session_key_chars"] == len("object-secret-session-key-do-not-forward")
assert object_approval_requested["payload"]["has_session_key"] is True
assert object_approval_requested["payload"]["description_chars"] == len("RAW_OBJECT_APPROVAL_DESCRIPTION_DO_NOT_FORWARD")
assert object_approval_requested["payload"]["command_chars"] == len("RAW_OBJECT_APPROVAL_COMMAND_DO_NOT_FORWARD")
subagent_finished = next(payload for payload in payloads if payload["type"] == "hermes.observer.subagent.finished")
assert "child_status" not in subagent_finished["payload"]
assert subagent_finished["payload"]["child_status_chars"] == len("done with RAW_CHILD_STATUS_DO_NOT_FORWARD")

count_before_disabled = len(events)
os.environ["HERMESHIP_OBSERVER_DISABLED"] = "1"
assert ctx.hooks["pre_tool_call"]({{"session_id": "disabled"}}) is None
assert len(events) == count_before_disabled

def failing_urlopen(request, timeout=None):
    raise OSError("RAW_FAIL_OPEN_SECRET_DO_NOT_FORWARD")

module.urllib.request.urlopen = failing_urlopen
os.environ.pop("HERMESHIP_OBSERVER_DISABLED", None)
assert ctx.hooks["pre_tool_call"]({{"session_id": "fail-open"}}) is None
"#,
            plugin_path = observer_plugin_init_path().display().to_string()
        );
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "observer plugin smoke failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("RAW_FAIL_OPEN_SECRET"),
            "fail-open diagnostic leaked raw exception detail"
        );
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
        observer_plugin_manifest: Option<&'static str>,
        observer_plugin_init: Option<&'static str>,
        operations: Option<&'static str>,
    }

    fn write_project_fixture(root: &Path, overrides: Option<ProjectFixtureOverrides>) {
        let overrides = overrides.unwrap_or_default();
        write(root.join("Cargo.toml"), CARGO_TOML_SAMPLE);
        write(root.join("Cargo.lock"), CARGO_LOCK_SAMPLE);
        write(
            root.join("README.md"),
            overrides.readme.unwrap_or(
                "hermeship setup\nhermeship install\nhermeship uninstall\nhermeship hermes install-plugin\nhermeship hermes enable-plugin\nhermeship git commit\nhermeship git branch-changed\nhermeship github issue-opened\nhermeship github pr-opened\nhermeship github check-failed\nhermeship github release-published\nhermeship tmux keyword\nhermeship tmux stale\nhermeship tmux watch\nhermeship tmux list\nhermeship cron run\nhermeship memory init\nhermeship memory status\nhermeship release preflight <version>\n",
            ),
        );
        write(
            root.join("docs/operations.md"),
            overrides.operations.unwrap_or(
                "hermeship setup --discord-token-stdin --default-channel ops\nhermeship install\nhermeship uninstall --remove-state --remove-config --remove-hooks --hermes-home ~/.hermes\nhermeship hermes install-plugin --home ~/.hermes\nhermeship hermes enable-plugin --home ~/.hermes --dry-run\nhermeship release preflight 0.1.0\n",
            ),
        );
        write(
            root.join("tests/fixtures/README.md"),
            "Fixtures must be synthetic and must not contain real tokens, cookies, secrets, full prompts, full conversations, or provider request/response bodies.\n",
        );
        write(
            root.join("tests/fixtures/cli/public_commands.txt"),
            overrides.public_commands.unwrap_or(
                "start\nstatus\nsetup --default-channel ops\nsend --channel ops --message hello\nemit hermes.agent.started --payload '{}'\nexplain hermes.agent.started --payload '{}'\nconfig show\nconfig path\nconfig verify\nhermes hook --payload '{}'\nhermes install-hooks --scope global --force\nhermes uninstall-hooks --dry-run\nhermes install-plugin --home /tmp/hermes --dry-run --force\nhermes enable-plugin --home /tmp/hermes --dry-run\ngit commit --repo hermeship --branch main --commit 1234567890abcdef1234567890abcdef12345678 --summary ship\ngit branch-changed --repo hermeship --old-branch main --new-branch codex/milestone-8-git\ngithub issue-opened --owner posp --repo hermeship --number 42 --title issue\ngithub pr-opened --owner posp --repo hermeship --number 17 --title pr --branch codex/milestone-8-github\ngithub check-failed --owner posp --repo hermeship --workflow ci --status failure --branch main\ngithub release-published --owner posp --repo hermeship --tag v0.1.0\ntmux keyword --session hermes-agent --keyword FAILED --line failed\ntmux stale --session hermes-agent --pane %2 --minutes 15 --last-line waiting\ntmux watch --session hermes-agent --keywords FAILED,complete --stale-minutes 10 --tmux-output 'hermes-agent\tmain\t%1\t0\tbash\tready'\ntmux list --tmux-output 'hermes-agent\tmain\t%1\t0\tbash\tready'\ncron run dev-followup\nmemory init --root /tmp/hermeship-memory --project Hermeship --date 2026-06-17\nmemory status --root /tmp/hermeship-memory --project Hermeship --date 2026-06-17\ninstall\nuninstall\nrelease preflight 0.1.0\n",
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
            root.join("templates/hermes-plugin/plugin.yaml"),
            overrides
                .observer_plugin_manifest
                .unwrap_or(OBSERVER_PLUGIN_MANIFEST_TEMPLATE),
        );
        write(
            root.join("templates/hermes-plugin/__init__.py"),
            overrides
                .observer_plugin_init
                .unwrap_or(OBSERVER_PLUGIN_INIT_TEMPLATE),
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

    fn observer_plugin_init_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/hermes-plugin/__init__.py")
    }
}
