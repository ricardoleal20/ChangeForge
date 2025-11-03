/// ================================ ///
///         OPTIONS :: List          ///
/// ================================ ///
use colored::*;
use std::collections::HashSet;
use terminal_size::{terminal_size, Width};
// Local imports
use crate::utilities::{find_largest_version, get_current_changesets};

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

pub fn list_changesets() {
    // Get the changesets and list them
    let changesets = get_current_changesets();
    // Header message with count
    let count = changesets.len();
    print_separator();
    println!(
        "📊 {}",
        format!(
            "Generating visualization of changes based on {} changesets read",
            count
        )
        .bright_black()
    );
    print_separator();
    // Find the current project version
    let new_version = find_largest_version(&changesets).unwrap();
    // Print the new version to set with these changesets
    println!("# New version to be bumped: v{}.\n", new_version.blue());
    // Add a vec of tags that were visited
    let mut printed_tags: HashSet<&String> = HashSet::new();
    // Process them
    for change_type in ["MAJOR", "MINOR", "PATCH"] {
        // Print the first layer

        // Filter the changesets for all those that match the change type
        for changeset in changesets.iter().filter(|c| c.change.trim() == change_type) {
            // If this tag has been printed already, then continue
            if printed_tags.contains(&changeset.tag) {
                continue;
            }
            // Print the tag if has not been printed already
            println!("- [{}]", changeset.tag.green());
            // And now, filter all the changesets for the tag and the change type
            for nested_changeset in changesets
                .iter()
                .filter(|c| c.change.trim() == change_type && c.tag == changeset.tag)
            {
                // If this changeset has a module, include it. If not, then just don't
                if nested_changeset.modules.is_empty() {
                    println!("    - {}", nested_changeset.message);
                } else {
                    println!(
                        "    - {}: {}",
                        nested_changeset.modules.blue(),
                        nested_changeset.message
                    );
                }
            }
            // And include this tag in the printed ones
            printed_tags.insert(&changeset.tag);
        }
    }
    print_separator();
}
