/// Just write the Changeset structure
use std::cmp::Ordering;

fn update_version(change: &str, version: String) -> String {
    let mut new_version = String::new();
    // We're going to update the version based on the change type
    match change {
        "MAJOR" => new_version = format!("{}.0.0", version.split('.').next().unwrap()),
        "MINOR" => {
            new_version = format!(
                "{}.{}.0",
                version.split('.').next().unwrap(),
                version.split('.').nth(1).unwrap()
            )
        }
        "PATCH" => {
            new_version = format!(
                "{}.{}.{}",
                version.split('.').next().unwrap(),
                version.split('.').nth(1).unwrap(),
                version.split('.').nth(2).unwrap()
            )
        }
        _ => {}
    }
    // Return the new version
    new_version
}

/// Changeset structure, including all the necessary fields
/// to process and create the new CHANGELOG.md
#[derive(Debug, Eq)]
pub struct Changeset {
    pub name: String,
    pub change: String,
    pub modules: String,
    pub tag: String,
    pub message: String,
    pub version: String,
}

impl Changeset {
    pub fn new(
        name: String,
        change: String,
        modules: String,
        tag: String,
        message: String,
        version: String,
    ) -> Changeset {
        // Get the new and updated version!
        let new_version = update_version(&change, version);
        // Initialize the changeset with the new version and the normal
        // methods
        Changeset {
            name,
            change,
            modules,
            tag,
            message,
            version: new_version,
        }
    }
}

// Implement the PartialEq to compare changesets between them
impl PartialEq for Changeset {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.change == other.change
            && self.modules == other.modules
            && self.tag == other.tag
            && self.message == other.message
            && self.version == other.version
    }
}

// Implement a PartialOrd method to sort the classes
impl PartialOrd for Changeset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Changeset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}
