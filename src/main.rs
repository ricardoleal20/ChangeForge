/// Here' we'll only import the CLI structure
/// inside of main so it can be used when it's called
use clap::{ArgMatches, Command};
// Local imports
mod options;
mod utilities;
// Use the methods from the modules
use options::{bump_version, create_changesets, init_project, list_changesets};
use utilities::create_subcommands;

fn main() {
    // Create the methods
    let (init, create, list, bump) = create_subcommands();
    // Instance the App
    let mut app = Command::new("ChangeForge :: Project management with Changesets")
        .subcommand(init)
        .subcommand(create)
        .subcommand(list)
        .subcommand(bump)
        .long_about("This module allows you to easily create and manage changesets for your project, providing a structured approach to documenting and tracking changes throughout the development process. Changesets help teams maintain better control over project updates, ensuring clear communication and effective collaboration. With this tool, you can streamline the process of recording changes, facilitating smoother project management and development workflows.")
        .about("Module for creating and using changesets to manage changes in team projects.

[Commands]
\t- `create`: Create a new changeset
\t- `list`: List the current changes and how they affect the current version
\t- `bump`: Release the new version and new changelog. Delete all the current changesets."
        );
    // Add the methods to the app method
    // Search for the matches
    let matches: ArgMatches = app.clone().get_matches();
    // Search for the matches
    match matches.subcommand() {
        // Init
        Some(("init", _)) => init_project(),
        // Create
        Some(("create", _)) => {
            // Instance the app method
            create_changesets()
        }
        // List
        Some(("list", _)) => {
            // Instance the app method
            list_changesets()
        }
        // Bump
        Some(("bump", _)) => {
            // Instance the app method
            bump_version()
        }
        _ => {
            // Manage the default cases for this project
            println!("{}", app.render_usage());
        }
    }
}
