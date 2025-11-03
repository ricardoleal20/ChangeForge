use colored::Colorize;
/// ================================ ///
///          OPTIONS :: Init          ///
/// ================================ ///
use inquire::{Confirm, MultiSelect, Text};
use std::fs;
use std::path::Path;

fn ask_bool(message: &str, default: bool) -> bool {
    Confirm::new(message)
        .with_default(default)
        .prompt()
        .expect("Error asking question")
}

fn ask_input(message: &str, default: &str) -> String {
    Text::new(message)
        .with_default(default)
        .prompt()
        .expect("Error asking input")
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
    println!("\n----------------------------------------\n");
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
                if let (Some(ext), Some(name)) = (path.extension(), path.file_name()) {
                    if ext == "toml" {
                        let n = name.to_string_lossy().to_string();
                        discovered.insert(n);
                    }
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
    loop {
        let remaining: Vec<String> = candidates
            .iter()
            .filter(|c| !selected.contains(*c))
            .cloned()
            .collect();
        let msg = format!(
            "{}",
            "Select the files for extracting and modifying the version paths"
        );
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
            Err(_) => {}
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
) -> String {
    let joined = version_paths
        .iter()
        .map(|p| format!("\"{}\"", p))
        .collect::<Vec<String>>()
        .join(", ");
    format!(
        "[changeforge]\nversion_path = [{}]\nchangesets_dir = \"{}\"\nchangelog_path = \"{}\"\n",
        joined, changesets_dir, changelog_path
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

pub fn init_project() {
    // 1) Select version files
    let version_paths = select_version_paths();
    print_separator();

    // General config paths
    let changesets_dir = ask_input("Changesets directory:", ".changesets");
    let changelog_path = ask_input("Changelog path:", "CHANGELOG.md");

    let config_content = generate_config_toml(&version_paths, &changesets_dir, &changelog_path);
    write_file_if_absent("changeforge.toml", &config_content);

    print_separator();
    // 2) Ask for PR workflow
    let add_pr_wf = ask_bool(
        "2) Add GitHub Workflow to manage the changes automatically on push?",
        true,
    );
    if add_pr_wf {
        let watch_branch = ask_input("    Branch for watch changes:", "bump-new-version");
        let base_branch = ask_input("    Base branch for the PR:", "main");
        let wf_content = generate_workflow_open_pr(&watch_branch, &base_branch);
        write_file_if_absent(".github/workflows/open_pr_on_push.yml", &wf_content);
    }

    print_separator();
    // 3) Ask for Release workflow
    let add_release_wf = ask_bool(
        "3) Add GitHub Workflow to create Release on target branch?",
        true,
    );
    if add_release_wf {
        let target_branch = ask_input("    Target branch for Releases:", "main");
        let wf_content = generate_workflow_auto_release(&target_branch);
        write_file_if_absent(".github/workflows/auto_release.yml", &wf_content);
    }

    println!("Initialized ChangeForge configuration.");
}
