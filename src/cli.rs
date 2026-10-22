use crate::{
    changeset::{self, Bump, Changeset},
    config::{self, Mode, RELEASE_FILE},
    release::{self, Plan},
    transaction::Lock,
};
use anyhow::{ensure, Context, Result};
use clap::{Parser, Subcommand};
use std::{
    collections::BTreeMap,
    fs,
    io::IsTerminal,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(
    name = "releaseweave",
    version,
    about = "Changesets for single repositories and monorepos. Built for GitHub."
)]
struct Cli {
    /// Repository root (defaults to the current directory).
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Discover packages and write configuration without overwriting existing files.
    Init {
        #[arg(short = 'y', long)]
        yes: bool,
        #[arg(long, value_enum)]
        mode: Option<Mode>,
        /// Install GitHub release-PR and release-on-merge workflow templates.
        #[arg(long)]
        github: bool,
    },
    /// Record a changeset, interactively or with flags for automation.
    #[command(alias = "add")]
    Create {
        #[arg(short = 'p', long = "package")]
        packages: Vec<String>,
        #[arg(short = 'b', long, value_enum)]
        bump: Option<Bump>,
        #[arg(short = 'm', long)]
        message: Option<String>,
        #[arg(long)]
        name: Option<String>,
    },
    /// Show the pending release plan. No files are modified.
    #[command(aliases = ["list", "status"])]
    Plan {
        #[arg(long)]
        json: bool,
    },
    /// Apply the release plan and consume only its changesets.
    #[command(alias = "version")]
    Bump {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Inspect the last applied release manifest for GitHub; never publishes.
    Release {
        #[arg(long)]
        json: bool,
        /// Explicitly document a local preview (release is always read-only).
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let root = cli
        .root
        .canonicalize()
        .context("Repository root does not exist")?;
    ensure!(root.is_dir(), "--root must point to a directory");
    match cli.command {
        Action::Init { yes, mode, github } => init(&root, yes, mode, github),
        Action::Create {
            packages,
            bump,
            message,
            name,
        } => {
            let _lock = Lock::acquire(&root)?;
            let cfg = config::load(&root)?;
            let mut packages = packages;
            if packages.is_empty() {
                if cfg.mode == Mode::Single {
                    packages.push(cfg.packages[0].name.clone());
                } else {
                    terminal("Select packages with --package NAME")?;
                    packages = inquire::MultiSelect::new(
                        "Packages affected:",
                        cfg.packages.iter().map(|p| p.name.clone()).collect(),
                    )
                    .prompt()?;
                }
            }
            let bump = match bump {
                Some(bump) => bump,
                None => {
                    terminal("Pass --bump patch|minor|major")?;
                    match inquire::Select::new("Version bump:", vec!["patch", "minor", "major"])
                        .prompt()?
                    {
                        "major" => Bump::Major,
                        "minor" => Bump::Minor,
                        _ => Bump::Patch,
                    }
                }
            };
            let message = match message {
                Some(message) => message,
                None => {
                    terminal("Pass --message TEXT")?;
                    inquire::Text::new("Describe the change:").prompt()?
                }
            };
            let change = Changeset {
                message,
                releases: packages.into_iter().map(|p| (p, bump)).collect(),
            };
            // Validate existing manifests too, so a broken config never produces a changeset.
            release::plan(&root, &cfg)?;
            let path = changeset::create(&root, &cfg, change, name)?;
            println!("Created {path}");
            Ok(())
        }
        Action::Plan { json } => {
            let cfg = config::load(&root)?;
            show(&release::plan(&root, &cfg)?, json)
        }
        Action::Bump { dry_run, json } => {
            let _lock = if dry_run {
                None
            } else {
                Some(Lock::acquire(&root)?)
            };
            let cfg = config::load(&root)?;
            let plan = release::plan(&root, &cfg)?;
            if !dry_run {
                release::bump(&root, &cfg, &plan)?;
            }
            show(&plan, json)?;
            if !json && !plan.releases.is_empty() {
                println!(
                    "{}",
                    if dry_run {
                        "Dry run: no files changed."
                    } else {
                        "Applied. GitHub release metadata: .releaseweave/release.json"
                    }
                );
            }
            Ok(())
        }
        Action::Release { json, dry_run: _ } => {
            let cfg = config::load(&root)?;
            let plan: Plan = serde_json::from_str(
                &fs::read_to_string(config::safe_path(&root, RELEASE_FILE)?)
                    .context("No applied release manifest. Run bump first")?,
            )?;
            ensure!(
                plan.schema_version == 1 && plan.mode == cfg.mode,
                "Release manifest schema/mode does not match config"
            );
            for release in &plan.releases {
                let package = cfg
                    .packages
                    .iter()
                    .find(|p| p.name == release.package)
                    .context("Unknown package in release metadata")?;
                ensure!(
                    crate::manifest::read(
                        &root,
                        &package.manifest,
                        package.version_key.as_deref()
                    )?
                    .version
                        == release.next_version,
                    "Release manifest is stale for {}",
                    package.name
                );
                let tag = match cfg.mode {
                    Mode::Single => format!("v{}", release.next_version),
                    Mode::Monorepo => format!("{}/v{}", package.name, release.next_version),
                };
                ensure!(tag == release.tag, "Invalid tag in release metadata");
            }
            show(&plan, json)?;
            if !json {
                println!("Preview only. The GitHub workflow publishes these tags after the release PR is merged.");
            }
            Ok(())
        }
    }
}

fn terminal(hint: &str) -> Result<()> {
    ensure!(
        std::io::stdin().is_terminal(),
        "Interactive input requires a terminal. {hint}"
    );
    Ok(())
}

fn show(plan: &Plan, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(plan)?);
    } else if plan.releases.is_empty() {
        println!("No pending changesets. Nothing to release.");
    } else {
        println!(
            "ReleaseWeave · {:?} · {} changeset(s)\n",
            plan.mode,
            plan.changesets.len()
        );
        println!("{:<28} {:<16} {:<16} TAG", "PACKAGE", "CURRENT", "NEXT");
        for release in &plan.releases {
            println!(
                "{:<28} {:<16} {:<16} {}",
                release.package,
                release.current_version.to_string(),
                release.next_version.to_string(),
                release.tag
            );
        }
        for release in &plan.releases {
            println!(
                "\n{} → {}\n{}",
                release.package, release.changelog, release.notes
            );
        }
    }
    Ok(())
}

fn init(root: &Path, yes: bool, mode: Option<Mode>, github: bool) -> Result<()> {
    let _lock = Lock::acquire(root)?;
    let existing = root.join("releaseweave.toml").exists();
    let cfg = if existing || root.join("changeforge.toml").exists() {
        config::load(root)?
    } else {
        config::discover(root, mode)?
    };
    if let Some(mode) = mode {
        ensure!(
            mode == cfg.mode,
            "Existing configuration has a different mode; edit releaseweave.toml explicitly"
        );
    }
    release::plan(root, &cfg)?;
    if !yes {
        terminal("Pass init --yes to accept discovered packages")?;
        for package in &cfg.packages {
            println!("{}: {}", package.name, package.manifest);
        }
        ensure!(
            inquire::Confirm::new("Save this configuration?")
                .with_default(true)
                .prompt()?,
            "Initialization canceled"
        );
    }
    let mut writes = BTreeMap::new();
    if !existing {
        writes.insert(
            "releaseweave.toml".into(),
            toml::to_string_pretty(&cfg)?.into_bytes(),
        );
    }
    if github {
        for (path, content) in [
            (
                ".github/workflows/releaseweave-pr.yml",
                include_str!("../templates/releaseweave-pr.yml"),
            ),
            (
                ".github/workflows/releaseweave-release.yml",
                include_str!("../templates/releaseweave-release.yml"),
            ),
        ] {
            if !config::safe_path(root, path)?.exists() {
                writes.insert(path.into(), content.as_bytes().to_vec());
            }
        }
    }
    crate::transaction::apply(root, writes, vec![])?;
    fs::create_dir_all(config::safe_path(root, &cfg.changesets_dir)?)?;
    println!(
        "ReleaseWeave initialized · {:?} · {} package(s)",
        cfg.mode,
        cfg.packages.len()
    );
    for package in &cfg.packages {
        println!("  {} → {}", package.name, package.manifest);
    }
    Ok(())
}
