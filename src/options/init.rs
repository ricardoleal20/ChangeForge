use colored::Colorize;
use inquire::error::InquireError;
use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};
/// ================================ ///
///          OPTIONS :: Init          ///
/// ================================ ///
use inquire::{set_global_render_config, Confirm, MultiSelect, Text};
use std::fs;
use std::path::Path;
use terminal_size::{terminal_size, Width};

fn ask_bool(message: &str, default: bool) -> bool {
    Confirm::new(message)
        .with_default(default)
        .prompt()
        .unwrap_or_else(|e| handle_cancel(e))
}

fn ask_input(message: &str, default: &str) -> String {
    Text::new(message)
        .with_default(default)
        .prompt()
        .unwrap_or_else(|e| handle_cancel(e))
}

fn write_file_if_absent(path: &str, content: &str) {
    if Path::new(path).exists() {
        return;
    }
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }
    fs::write(path, content).expect("Error writing file");
}

fn print_separator() {
    let default_width: usize = 60;
    let width: usize = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(default_width)
        .saturating_sub(2)
        .clamp(40, 100);
    let line = "─".repeat(width);
    println!("\n{}\n", line.bright_black());
}

fn print_note(message: &str) {
    println!("{}", message.bright_black());
}

fn print_success(message: &str) {
    let default_width: usize = 60;
    let width: usize = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(default_width)
        .clamp(40, 100);
    let inner = width.saturating_sub(2);
    let top = format!(
        "{}{}{}",
        "┌".bright_black(),
        "─".repeat(inner).bright_black(),
        "┐".bright_black()
    );
    let bottom = format!(
        "{}{}{}",
        "└".bright_black(),
        "─".repeat(inner).bright_black(),
        "┘".bright_black()
    );
    let content = format!(" {} {} ", "✔".green().bold(), message.green());
    let len = content.chars().count();
    let pad_total = inner.saturating_sub(len);
    let pad_left = pad_total / 2;
    let pad_right = pad_total.saturating_sub(pad_left);
    let middle = format!(
        "{}{}{}{}{}",
        "│".bright_black(),
        " ".repeat(pad_left),
        content,
        " ".repeat(pad_right),
        "│".bright_black()
    );
    println!("\n{}\n{}\n{}\n", top, middle, bottom);
}

fn print_cancel(message: &str) {
    let default_width: usize = 60;
    let width: usize = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(default_width)
        .clamp(40, 100);
    let inner = width.saturating_sub(2);
    let top = format!(
        "{}{}{}",
        "┌".bright_black(),
        "─".repeat(inner).bright_black(),
        "┐".bright_black()
    );
    let bottom = format!(
        "{}{}{}",
        "└".bright_black(),
        "─".repeat(inner).bright_black(),
        "┘".bright_black()
    );
    let content = format!(" {} {} ", "✖".red().bold(), message.red());
    let len = content.chars().count();
    let pad_total = inner.saturating_sub(len);
    let pad_left = pad_total / 2;
    let pad_right = pad_total.saturating_sub(pad_left);
    let middle = format!(
        "{}{}{}{}{}",
        "│".bright_black(),
        " ".repeat(pad_left),
        content,
        " ".repeat(pad_right),
        "│".bright_black()
    );
    println!("\n{}\n{}\n{}\n", top, middle, bottom);
}

fn handle_cancel(err: InquireError) -> ! {
    match err {
        InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            print_cancel("Operation canceled by user");
            std::process::exit(130);
        }
        other => {
            // Unexpected error: show message and exit with failure
            print_cancel(&format!("Error: {}", other));
            std::process::exit(1);
        }
    }
}

fn apply_inquire_theme() {
    let rc = RenderConfig {
        prompt_prefix: Styled::new("❯"),
        answered_prompt_prefix: Styled::new("✔"),
        ..RenderConfig::default()
    };
    set_global_render_config(rc);
}

fn sub_prompt_render_config() -> RenderConfig {
    RenderConfig {
        prompt_prefix: Styled::new("↳"),
        answered_prompt_prefix: Styled::new("•"),
        prompt: StyleSheet::new().with_fg(Color::LightBlue),
        answer: StyleSheet::new().with_fg(Color::LightGreen),
        help_message: StyleSheet::new().with_fg(Color::DarkGrey),
        ..RenderConfig::default()
    }
}

fn select_version_paths() -> Vec<String> {
    use std::collections::HashSet;
    let mut selected: Vec<String> = Vec::new();
    // Discover TOML files in current directory
    let mut discovered: HashSet<String> = HashSet::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let (Some(_), Some(name)) = (path.extension(), path.file_name()) {
                    let n = name.to_string_lossy().to_string();
                    discovered.insert(n);
                }
            }
        }
    }
    // Prioritize common files
    let mut candidates: Vec<String> = Vec::new();
    for p in ["pyproject.toml", "Cargo.toml"] {
        if Path::new(p).exists() {
            candidates.push(p.to_string());
            discovered.remove(p);
        }
    }
    // Append other discovered TOML files
    let mut others: Vec<String> = discovered.into_iter().collect();
    others.sort();
    candidates.extend(others);

    // MultiSelect for discovered
    // One-time description of what this selection does
    print_note("These files will be used to extract and update the project version during bumps.");

    loop {
        let remaining: Vec<String> = candidates
            .iter()
            .filter(|c| !selected.contains(*c))
            .cloned()
            .collect();
        let msg = "Select the files for extracting and modifying the version paths".to_string();
        let choices = remaining.clone();
        let picked = MultiSelect::new(&msg, choices)
            .with_help_message("Use arrows/space to select, enter to confirm")
            .prompt();
        match picked {
            Ok(items) => {
                for it in items {
                    if !selected.contains(&it) {
                        selected.push(it);
                    }
                }
            }
            Err(e) => handle_cancel(e),
        }
        if !selected.is_empty() {
            break;
        }
        println!("Please select at least one file or specify a path.");
        // If none selected, allow manual path input
        let path = ask_input("    Enter file path:", "");
        if !path.trim().is_empty() && !selected.contains(&path) {
            selected.push(path);
        }
        if !selected.is_empty() {
            break;
        }
    }

    // Allow manual additions after selection
    loop {
        let add_more = ask_bool(
            &format!("{}", "    Add another file path manually?".bright_black()),
            false,
        );
        if !add_more {
            break;
        }
        let path = ask_input("    Enter file path:", "");
        if !path.trim().is_empty() && !selected.contains(&path) {
            selected.push(path);
        }
    }
    selected
}

fn generate_config_toml(
    version_paths: &[String],
    changesets_dir: &str,
    changelog_path: &str,
    ai_enabled: bool,
    templates_dir: &str,
    commit_on_create: bool,
) -> String {
    let joined = version_paths
        .iter()
        .map(|p| format!("\"{}\"", p))
        .collect::<Vec<String>>()
        .join(", ");
    format!(
        "[changeforge]\nversion_path = [{}]\nchangesets_dir = \"{}\"\nchangelog_path = \"{}\"\nai_enabled = {}\ntemplates_dir = \"{}\"\ncommit_on_create = {}\n",
        joined, changesets_dir, changelog_path, ai_enabled, templates_dir, commit_on_create
    )
}

fn generate_workflow_open_pr(branch_watch: &str, base_branch: &str) -> String {
    format!(
        "name: Open PR on push\n\non:\n  push:\n    branches:\n      - {}\n\njobs:\n  open_pr:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: actions/setup-python@v5\n        with:\n          python-version: '3.10'\n      - name: Install PyGithub\n        run: pip install PyGithub\n      - name: Open PR\n        env:\n          GITHUB_TOKEN: ${{{{ secrets.GITHUB_TOKEN }}}}\n          REPO_NAME: ${{{{ github.repository }}}}\n          BRANCH_NAME: {}\n          HEAD_BRANCH: ${{{{ github.ref_name }}}}\n        run: python .github/utilities/open_pr.py\n",
        branch_watch, base_branch
    )
}

fn generate_workflow_auto_release(target_branch: &str) -> String {
    format!(
        "name: Auto Release on target branch\n\non:\n  push:\n    branches:\n      - {}\n\njobs:\n  release:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - name: Extract version\n        id: v\n        shell: bash\n        run: |\n          set -e\n          ver=\"\"\n          if [ -f pyproject.toml ]; then\n            ver=$(grep -m1 -oE 'version\\s*=\\s*\"[0-9]+\\.[0-9]+\\.[0-9]+\"' pyproject.toml | sed -E 's/.*\"([0-9.]+)\"/\\1/')\n          fi\n          if [ -z \"$ver\" ] && [ -f Cargo.toml ]; then\n            ver=$(grep -m1 -oE '^version\\s*=\\s*\"[0-9]+\\.[0-9]+\\.[0-9]+\"' Cargo.toml | sed -E 's/.*\"([0-9.]+)\"/\\1/')\n          fi\n          echo \"tag=v$ver\" >> $GITHUB_OUTPUT\n      - name: Create GitHub Release\n        uses: softprops/action-gh-release@v1\n        with:\n          tag_name: ${{{{ steps.v.outputs.tag }}}}\n          name: ${{{{ steps.v.outputs.tag }}}}\n          body_path: CHANGELOG.md\n        env:\n          GITHUB_TOKEN: ${{{{ secrets.GITHUB_TOKEN }}}}\n",
        target_branch
    )
}

fn ask_creation_options() -> (bool, bool) {
    print_note("Add the following options for creation of changesets:");
    let options = vec![
        "🤖 AI messages: Allow generating messages with AI during create (requires an API key)"
            .to_string(),
        "💾 Commit after create: Ask to commit the changeset and selected files".to_string(),
    ];
    let selected = MultiSelect::new(
        "Use arrows/space to select, enter to confirm",
        options.clone(),
    )
    .with_help_message("Select none, one, or many options")
    .prompt()
    .unwrap_or_else(|e| handle_cancel(e));
    let ai_enabled = selected.iter().any(|s| s.contains("AI messages"));
    let commit_on_create = selected.iter().any(|s| s.contains("Commit after create"));
    (ai_enabled, commit_on_create)
}

pub fn init_project() {
    // Apply theme once for this session
    apply_inquire_theme();
    // 1) Select version files
    let version_paths = select_version_paths();
    print_separator();

    // General config paths
    print_note("Default paths will be written to changeforge.toml (editable later).");
    let changesets_dir = ".changesets";
    let changelog_path = "CHANGELOG.md";

    print_separator();
    // Extra options (single multi-select for bools)
    let (ai_enabled, commit_on_create) = ask_creation_options();
    print_separator();
    print_note("Select or create a folder for message templates (optional).");
    let templates_dir = Text::new("Templates directory (leave empty to disable):")
        .with_default("")
        .prompt()
        .unwrap_or_else(|e| handle_cancel(e));
    if !templates_dir.trim().is_empty() {
        let _ = fs::create_dir_all(&templates_dir);
    }

    let config_content = generate_config_toml(
        &version_paths,
        changesets_dir,
        changelog_path,
        ai_enabled,
        templates_dir.trim(),
        commit_on_create,
    );
    write_file_if_absent("changeforge.toml", &config_content);

    print_separator();
    // 2) Ask for PR workflow
    print_note("On pushes to the watched branch, a PR will be opened to the base branch.");
    let add_pr_wf = ask_bool(
        "Add GitHub Workflow to manage the changes automatically on push?",
        true,
    );
    if add_pr_wf {
        let sub_rc = sub_prompt_render_config();
        let watch_branch = Text::new("    Branch for watch changes:")
            .with_default("bump-new-version")
            .with_render_config(sub_rc)
            .prompt()
            .unwrap_or_else(|e| handle_cancel(e));
        let base_branch = Text::new("    Base branch for the PR:")
            .with_default("main")
            .with_render_config(sub_prompt_render_config())
            .prompt()
            .unwrap_or_else(|e| handle_cancel(e));
        let wf_content = generate_workflow_open_pr(&watch_branch, &base_branch);
        write_file_if_absent(".github/workflows/open_pr_on_push.yml", &wf_content);
    }

    print_separator();
    // 3) Ask for Release workflow
    print_note(
        "On pushes to the target branch, a GitHub Release will be created from CHANGELOG.md.",
    );
    let add_release_wf = ask_bool(
        "Add GitHub Workflow to create Release on target branch?",
        true,
    );
    if add_release_wf {
        let sub_rc = sub_prompt_render_config();
        let target_branch = Text::new("    Target branch for Releases:")
            .with_default("main")
            .with_render_config(sub_rc)
            .prompt()
            .unwrap_or_else(|e| handle_cancel(e));
        let wf_content = generate_workflow_auto_release(&target_branch);
        write_file_if_absent(".github/workflows/auto_release.yml", &wf_content);
    }

    print_success("Initialized ChangeForge configuration.");
}
