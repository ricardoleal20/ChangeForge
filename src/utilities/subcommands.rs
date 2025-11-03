/// ================================ ///
///      UTILITIES :: SubCommand     ///
/// ================================ ///
use clap::Command;

// create a type APP to avoid write it on every command
type CLIApp = Command;

fn add_init_subcommand() -> CLIApp {
    let init_subcommand: CLIApp = Command::new("init")
        .about("Initialize ChangeForge configuration and optional GitHub workflows")
        .long_about("Create a standalone changeforge.toml configuration file and optionally add GitHub workflows to open PRs on push and to create Releases automatically.");
    init_subcommand
}

fn add_create_subcommand() -> CLIApp {
    // Here, create the subcommand `create`
    let create_subcommand: CLIApp = Command::new("create")
        .about("Create a new Changeset")
        .long_about("With a bunch of options, create the new changeset for a set of development");
    // Return the subcommand
    create_subcommand
}

fn add_list_subcommand() -> CLIApp {
    // Here, create the subcommand `list`
    let list_subcommand: CLIApp = Command::new("list")
        .about("List all the current Changesets")
        .long_about("List the current changesets and a short description from them. Doesn't require arguments.");
    // Return the subcommand
    list_subcommand
}

fn add_bump_subcommand() -> CLIApp {
    // Here, create the subcommand `bump`
    let bump_subcommand: CLIApp = Command::new("bump")
        .about("Bump the new version using the pending changesets")
        .long_about("Using all the current changesets that we have locally, and delete them after update the changelog and the version of the package.
        
It is also going to delete all the current files in the `.changesets` folder (to restart the process)");
    // Return the subcommand
    bump_subcommand
}

/// Create and append the subcommands  for the CLI application
pub fn create_subcommands() -> (CLIApp, CLIApp, CLIApp, CLIApp) {
    // Create the `create`` subcommand
    let init = add_init_subcommand();
    let create = add_create_subcommand();
    // Create the `list` subcommand
    let list = add_list_subcommand();
    // Create the `bump` subcommand
    let bump = add_bump_subcommand();
    // Create the extra commands
    // Return the commands
    (init, create, list, bump)
}
