use colored::*;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use terminal_size::{terminal_size, Width};
// Local imports
use crate::options::Changeset;
use crate::utilities::update_version_path;

/// Function to open the Changeset in case that exists
pub fn open_changelog() -> Vec<String> {
    // Open the Changeset file in case that exist
    let file = fs::File::open("CHANGELOG.md")
        .expect("Error opening CHANGELOG.md. Ensure that you have one already.");
    let reader = BufReader::new(file);

    // Create the content structure
    let mut content: Vec<String> = Vec::new();

    reader.lines().for_each(|line| {
        if let Ok(line_content) = line {
            content.push(line_content);
        }
    });
    // And return it
    content
}

pub fn create_changelog(content: Vec<String>, version: &str) {
    // Create a new CHANGELOG.md file
    let mut file = fs::File::create("CHANGELOG.md").expect("Error creating the CHANGELOG.md");

    // Write the entire CHANGELOG content
    writeln!(file, "{}", content.join("\n")).expect("Error when writing the CHANGELOG.md");
    // Write the new version file too
    update_version_path(version);
    // Delete all the current changesets
    delete_changesets();
    // If everything's cool, then write the successful message (styled)
    print_success_box("CHANGELOG.md and version updated!");
}

pub fn new_changelog_entry(changesets: &[Changeset], version: &String) -> Vec<String> {
    // Update the version based on the latest
    // First, get a list of printed tags to avoid read the same tag twice
    let mut printed_tags: HashSet<&String> = HashSet::new();
    // Create a mutable for the content written
    let mut content: Vec<String> = Vec::new();
    content.push(format!("## [{}]\n", version));
    for changeset in changesets.iter() {
        // Evaluate if this tag has been written
        if printed_tags.contains(&changeset.tag) {
            continue;
        }
        // Write the tag first
        content.push(format!("\n### {}\n\n", changeset.tag));
        // Filter for all the same tags
        for nested_changeset in changesets.iter().filter(|c| c.tag == changeset.tag) {
            // Then, write all the changes
            if nested_changeset.modules.is_empty() {
                content.push(format!("- {}.\n", nested_changeset.message));
            } else {
                content.push(format!(
                    "- {}: {}.\n",
                    nested_changeset.modules, nested_changeset.message
                ));
            }
        }
        // And at the end, write this tag on the read ones
        printed_tags.insert(&changeset.tag);
    }
    // And at the end, return the content list
    content
}

fn delete_changesets() {
    let folder_path = ".changesets";
    // Verify if the folder exist
    if let Ok(entries) = fs::read_dir(folder_path) {
        // Iterate over all the changesets in that folder
        for entry in entries.flatten() {
            let path = entry.path();
            // For security, verify if the entry is a file
            if path.is_file() {
                // Try to remove the file
                if let Err(e) = fs::remove_file(&path) {
                    // If you could not delete a file, then panic
                    panic!("Error deleting file {}: {}", path.display(), e);
                }
            }
        }
    } else {
        // In this case, panic. It should only reach to this function in case that
        // the folder `.changeset` exists
        panic!("The folder {} does not exist.", folder_path);
    }
}

fn print_success_box(message: &str) {
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
