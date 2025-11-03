/// ================================ ///
///         OPTIONS :: Create        ///
/// ================================ ///
/// For this, we'll follow the next path:
/// * P1: Set the changeset name (If not specified, would be randomly chosen)
/// * P2: Select the type of versioning change (major, minor, patch)
/// * P3: Search for the available modules in the package. If not found, let them write their own module name
/// * P4: Write the message to add in the changeset
use colored::*;
use fake::faker::lorem::en::Word;
use fake::Fake;
use inquire::error::InquireError;
use inquire::ui::{RenderConfig, Styled};
use inquire::{set_global_render_config, Confirm, Select, Text};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;
use terminal_size::{terminal_size, Width};
// Local imports
use crate::options::Changeset;
use crate::utilities::{
    create_changeset_folder, find_version, generate_ai_message, load_changeforge_config,
    version_operations::calculate_next_version, write_changeset_file, AIConfig,
};

/// Detect modules in the project by scanning files
fn detect_modules() -> Vec<String> {
    let mut modules = Vec::new();

    // Add common directories to scan
    let directories = vec!["src", "tests", "lib", "app"];

    for dir in directories {
        // Skip if directory doesn't exist
        if !Path::new(dir).exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(path) = entry.path().to_str() {
                            // Add the file path as a module
                            modules.push(path.to_string());
                        }
                    }
                }
            }
        }
    }

    // Add "Other" option to let user input custom module
    modules.push("Other (specify manually)".to_string());

    modules
}

// Get default message template based on change type and tag
// (unused) kept templates are now read from templates_dir

/// Get changed files from git
fn get_git_changed_files() -> Vec<String> {
    let mut changed_files = Vec::new();

    // Try to get modified files from git
    let output = Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let git_output = String::from_utf8_lossy(&output.stdout);
            for line in git_output.lines() {
                if !line.is_empty() {
                    changed_files.push(line.to_string());
                }
            }
        }
    }

    // Add "Other" option at the end
    if !changed_files.is_empty() {
        changed_files.push("Other (specify manually)".to_string());
    }

    changed_files
}

/// Read templates from configured directory
fn read_templates_from_dir(dir: &str) -> Vec<(String, String)> {
    let mut templates: Vec<(String, String)> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| String::from("template"));
                if let Ok(content) = fs::read_to_string(&path) {
                    templates.push((name, content));
                }
            }
        }
    }
    templates
}

/// Select tags depending on the change type
fn select_tags(change_type: &str) -> Vec<String> {
    let available_tags: Vec<String>;
    // Based on the change type representation, select the tags.
    if change_type == "MAJOR" {
        available_tags = vec![
            String::from("⚰️  Remove: Removed features."),
            String::from("🚚 Rename: Renamed features."),
            String::from("✏️  I/O: Changing input/output of features."),
            String::from("💥 Behavior: Changing features behavior."),
        ];
    } else if change_type == "MINOR" {
        available_tags = vec![
            String::from("✨ Feature: New feature."),
            String::from("➕ Add: Add functionality to existing feature."),
            String::from("✏️  I/O: Include optional input/output to a feature."),
            String::from("🗑️  Deprecated: Deprecated features."),
        ];
    } else {
        available_tags = vec![
            String::from("♻️  Refactor: Refactor of existing code."),
            String::from("🐛 Bug: Fix a bug."),
            String::from("⚡️ Optimization: Simple optimization of code."),
            String::from("🧪 Tests: Include or update tests."),
            String::from("🩹 Patch: Include or delete logs, catch errors or related things."),
        ];
    }
    // Return the selected tags
    available_tags
}

/// Create the question to set the tag
fn set_tag(change_type: &str) -> (String, String) {
    // Get the available tags
    let available_tags = select_tags(change_type);
    let ans = Select::new("Select the tag for this change", available_tags)
        .prompt()
        .unwrap_or_else(|e| handle_cancel(e));
    let mut tag = ans.as_str();
    // Extract icon (first token) for commit usage
    let icon = ans.split_whitespace().next().unwrap_or("").to_string();
    // And now, clean the tag
    let re = Regex::new(r"([A-Za-z]+):").unwrap();
    if let Some(capture) = re.captures(tag) {
        if let Some(matched) = capture.get(1) {
            tag = matched.as_str();
        }
    }
    (tag.to_string(), icon)
}

/// Create the questions
fn create_name_and_type(default_name: &str) -> (String, String) {
    apply_inquire_theme();
    print_note("Provide a name and select the change type for this changeset.");
    let name = Text::new("Write the Changeset name")
        .with_default(default_name)
        .prompt()
        .unwrap_or_else(|e| handle_cancel(e));
    let change_type = Select::new(
        "Select the change type that is most adequate to these changes",
        vec![
            "💥 MAJOR: Most of the time related to breaking changes.".to_string(),
            "✨ MINOR: New features that keep backwards compatibility.".to_string(),
            "🩹 PATCH: Refactors, bugs, fixes and small changes.".to_string(),
        ],
    )
    .prompt()
    .unwrap_or_else(|e| handle_cancel(e));
    (name, change_type)
}

/// Ask for module based on git changes and auto-detected modules
fn ask_for_module() -> String {
    // First try to get git changed files
    let git_modules = get_git_changed_files();

    if !git_modules.is_empty() {
        let choice = Select::new("Select the module/file that has changed", git_modules)
            .prompt()
            .unwrap_or_else(|e| handle_cancel(e));
        if choice == "Other (specify manually)" {
            Text::new("Enter the custom module name")
                .with_default("")
                .prompt()
                .unwrap_or_else(|e| handle_cancel(e))
        } else {
            choice
        }
    } else {
        let detected_modules = detect_modules();
        if detected_modules.len() > 1 {
            let choice = Select::new("Select the module that has changed", detected_modules)
                .prompt()
                .unwrap_or_else(|e| handle_cancel(e));
            if choice == "Other (specify manually)" {
                Text::new("Enter the custom module name")
                    .with_default("")
                    .prompt()
                    .unwrap_or_else(|e| handle_cancel(e))
            } else {
                choice
            }
        } else {
            Text::new("Write the module/class/function name that has changed (optional)")
                .with_default("")
                .prompt()
                .unwrap_or_else(|e| handle_cancel(e))
        }
    }
}

/// Ask for message generation method (AI, template, manual)
fn ask_for_message_method() -> String {
    let cfg = load_changeforge_config();
    let mut options: Vec<String> = vec!["Write message from scratch".to_string()];
    // templates gating: require directory with at least one file
    if let Some(dir) = cfg.templates_dir.as_ref() {
        if let Ok(mut rd) = std::fs::read_dir(dir) {
            if rd.next().is_some() {
                options.insert(0, "Use message template".to_string());
            }
        }
    }
    // AI gating
    if cfg.ai_enabled {
        options.insert(0, "Generate with AI based on detected changes".to_string());
    }
    Select::new(
        "How would you like to create your changeset message?",
        options,
    )
    .prompt()
    .unwrap_or_else(|e| handle_cancel(e))
}

/// Ask for the message with template suggestions
fn ask_for_message(change_type: &str, tag: &str, module: &str) -> String {
    // First, ask which method to use
    let method = ask_for_message_method();

    if method.contains("Generate with AI") {
        // Create AI configuration using build method
        let config = AIConfig::build();

        // Generate a message with AI
        println!("Analyzing changes and generating message...");

        // We need to block on the async call since we're in a sync context
        let ai_message = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(generate_ai_message(change_type, tag, module, &config))
            .unwrap_or_else(|e| {
                println!("Error generating AI message: {}", e);
                "Error generating message".to_string()
            });

        // Ask if user wants to edit the generated message
        let edit = Confirm::new(&format!(
            "AI generated message: \n\"{}\"\n\nWould you like to edit this message?",
            ai_message
        ))
        .with_default(false)
        .prompt()
        .unwrap_or_else(|e| handle_cancel(e));

        if edit {
            // User wants to edit the message
            Text::new("Edit the message:")
                .with_default(&ai_message)
                .prompt()
                .unwrap_or_else(|e| handle_cancel(e))
        } else {
            // Use the AI message as is
            ai_message
        }
    } else if method.contains("Use message template") {
        // Use external templates from configured directory
        let cfg = load_changeforge_config();
        let dir = cfg.templates_dir.as_deref().unwrap_or("templates/messages");
        let templates = read_templates_from_dir(dir);
        if templates.is_empty() {
            // This should not happen due to gating, but fallback to manual entry
            return Text::new("Write the message for the change")
                .with_default("")
                .prompt()
                .unwrap_or_else(|e| handle_cancel(e));
        }

        let names: Vec<String> = templates.iter().map(|(n, _)| n.clone()).collect();
        let picked_name = Select::new("Select a template", names)
            .prompt()
            .unwrap_or_else(|e| handle_cancel(e));
        let template = templates
            .into_iter()
            .find(|(n, _)| n == &picked_name)
            .map(|(_, c)| c)
            .unwrap_or_default();

        let mut message: String = Text::new("Write the message for the change")
            .with_default(&template)
            .prompt()
            .unwrap_or_else(|e| handle_cancel(e));

        while message.is_empty() || message == template {
            println!(
                "Error: You need to add a personalized message. The template cannot be used as is."
            );
            message = Text::new("Write the message for the change (customize the template)")
                .with_default(&template)
                .prompt()
                .unwrap_or_else(|e| handle_cancel(e));
        }

        message.to_string()
    } else {
        // Write from scratch
        let message = Text::new("Write the message for the change")
            .with_default("")
            .prompt()
            .unwrap_or_else(|e| handle_cancel(e));

        if message.is_empty() {
            panic!("There was no message for the changeset. You need to add a message.");
        }

        message.to_string()
    }
}

/// Display a summary and confirm before saving
fn confirm_changeset(changeset: &Changeset) -> bool {
    // Pretty summary box
    print_summary_box(changeset);

    Confirm::new("Do you want to save this changeset?")
        .with_default(true)
        .prompt()
        .unwrap_or_else(|e| handle_cancel(e))
}

fn process_answers() -> Changeset {
    // Generate the default name
    let default_name = "Leave it blank for a random name";
    let (mut name, selected_change) = create_name_and_type(default_name);
    if name == default_name || name.trim().is_empty() {
        name = Word().fake();
    }
    let mut change = selected_change.as_str();
    // Instance the regex to search for the word
    let re = Regex::new(r"\b(MAJOR|MINOR|PATCH)\b").unwrap();
    // get the change
    if let Some(capture) = re.captures(change) {
        if let Some(matched) = capture.get(1) {
            change = matched.as_str();
        }
    }

    // Get the tag (now that we know the change type)
    let (tag, _tag_icon) = set_tag(change);

    // Get the module (with git and auto-detection)
    let module = ask_for_module();

    // Get the message (with AI, templates, or manual input)
    let message = ask_for_message(change, &tag, &module);

    // Get the current version
    let current_version = find_version();

    // Calculate the next version based on the change type
    let next_version = calculate_next_version(&current_version, change);

    // Create the changeset
    let changeset = Changeset {
        name,
        change: change.into(),
        modules: module,
        tag,
        message,
        version: next_version,
    };

    // Return the changeset only if confirmed
    if confirm_changeset(&changeset) {
        // Attach icon info via side channel by returning after commit stage
        // We'll perform commit in create_changesets where we still know module path
        // For now return the built changeset
        changeset
    } else {
        print_cancel("Operation canceled by user");
        std::process::exit(130);
    }
}

pub fn create_changesets() {
    // Process the results
    let changeset: Changeset = process_answers();
    // Then, start creating the Changeset file in the changeset function
    // Let's see if the folder exists. If not, create it
    create_changeset_folder();
    // Once you have created the folder, create the changeset
    write_changeset_file(&changeset);
    // Optional commit on create
    let cfg = load_changeforge_config();
    if cfg.commit_on_create {
        let do_commit = Confirm::new("Do you want to commit the changeset and selected files?")
            .with_default(true)
            .prompt()
            .unwrap_or_else(|e| handle_cancel(e));
        if do_commit {
            // Determine commit components
            // Build commit message icon + tag + user message
            // Re-select icon from the tag list using the stored tag description pattern
            // Best-effort: infer icon by mapping tags
            let icon = match changeset.tag.as_str() {
                "Remove" => "⚰️",
                "Rename" => "🚚",
                "I" => "✏️", // fallback, unlikely used
                "Behavior" => "💥",
                "Feature" => "✨",
                "Add" => "➕",
                "Deprecated" => "🗑️",
                "Refactor" => "♻️",
                "Bug" => "🐛",
                "Optimization" => "⚡️",
                "Tests" => "🧪",
                "Patch" => "🩹",
                _ => "🔖",
            };
            let commit_msg = format!("{} {}: {}", icon, changeset.tag, changeset.message);
            // Paths to add
            let mut paths: Vec<String> = Vec::new();
            paths.push(format!(".changesets/{}.toml", changeset.name));
            if !changeset.modules.is_empty() && Path::new(&changeset.modules).exists() {
                paths.push(changeset.modules.clone());
            }
            // Run git add and commit
            let _ = Command::new("git").args(["add"]).args(&paths).status();
            let _ = Command::new("git")
                .args(["commit", "-m", &commit_msg])
                .status();
        }
    }
    // Once you have created it, print a confirmation message
    print_success_box(&format!(
        "Changeset `{}` has been created!",
        format!("{}.toml", changeset.name).green()
    ));
}

// ===== Theming & helpers (similar approach to init) =====
fn apply_inquire_theme() {
    let rc = RenderConfig {
        prompt_prefix: Styled::new("❯"),
        answered_prompt_prefix: Styled::new("✔"),
        ..RenderConfig::default()
    };
    set_global_render_config(rc);
}

fn print_note(message: &str) {
    println!("{}", message.bright_black());
}

fn print_cancel(message: &str) {
    print_cancel_box(message);
}

fn handle_cancel(err: InquireError) -> ! {
    match err {
        InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            print_cancel("Operation canceled by user");
            std::process::exit(130);
        }
        other => {
            print_cancel(&format!("Error: {}", other));
            std::process::exit(1);
        }
    }
}

// ---------- Pretty boxes (separator/success/cancel/summary) ----------
fn print_box_lines(lines: &[String], accent: &str) {
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
    println!("\n{}", top);
    for line in lines {
        let content = format!(" {} {}", accent, line);
        let len = content.chars().count();
        let pad_total = inner.saturating_sub(len);
        let pad_right = pad_total; // left already included by accent + space
        println!(
            "{}{}{}{}",
            "│".bright_black(),
            content,
            " ".repeat(pad_right),
            "│".bright_black()
        );
    }
    println!("{}\n", bottom);
}

fn print_success_box(message: &str) {
    let line = format!("{} {}", "✔".green().bold(), message.green());
    print_box_lines(&[line], "");
}

fn print_cancel_box(message: &str) {
    let line = format!("{} {}", "✖".red().bold(), message.red());
    print_box_lines(&[line], "");
}

fn print_summary_box(changeset: &Changeset) {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("{}", "Changeset Summary:".bold()));
    lines.push(format!("Name: {}.toml", changeset.name));
    lines.push(format!("Type: {}", changeset.change));
    lines.push(format!("Tag: {}", changeset.tag));
    if !changeset.modules.is_empty() {
        lines.push(format!("Module: {}", changeset.modules));
    }
    lines.push(format!("Message: {}", changeset.message));
    lines.push(format!("Version: {}", changeset.version));
    print_box_lines(&lines, "");
}
