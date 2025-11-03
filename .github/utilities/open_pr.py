"""
Open a Pull Request for the latest versions bumped on the
main branch.
"""
import os
import re
from typing import Optional, List
# Import the Github token
from github import Github, InputGitTreeElement


def get_codeowners() -> List[str]:
    """Get the list of CODEOWNERS from the CODEOWNERS file."""
    codeowners = []
    codeowners_paths = [".github/CODEOWNERS", "docs/CODEOWNERS", "CODEOWNERS"]

    for path in codeowners_paths:
        if os.path.exists(path):
            with open(path, "r", encoding="utf-8") as f:
                for line in f:
                    # Skip comments and empty lines
                    if line.strip() and not line.startswith("#"):
                        # Extract usernames (those starting with @)
                        owners = re.findall(r"@(\w+)", line)
                        codeowners.extend(owners)

    # Remove duplicates while preserving order
    return list(dict.fromkeys(codeowners))


def get_all_file_paths(
    relative_path: str,
    available_files: Optional[set] = None,
    path: Optional[str] = None,
) -> set[tuple[str, str]]:
    """Get all the file paths available"""
    # If there's no available files, then instance one
    if available_files is None:
        available_files = set()
    # Iterate over all the files in this path
    for file in os.listdir(path):
        # Ignore those that are private or are related to a dump
        if file.startswith(".") or file.endswith(("target", ".png", ".jpeg")):
            # Make sure that the changesets are also reviewed,
            # so this is an exception of the `startswith(.)`
            if not file.startswith(".changesets"):
                continue
        path_file = os.path.join(path, file)
        # If this is a directory, search into it
        if os.path.isdir(path_file):
            available_files = get_all_file_paths(
                relative_path, available_files, path_file)
        else:
            # If not, add the current path
            with open(path_file, 'r', encoding="utf-8") as f:
                available_files.add(
                    (os.path.relpath(path_file, relative_path), f.read())
                )
    # At the end, return the set
    return available_files

def apply_changesets(token: str, repo: str, branch: str) -> None:
    """Apply the changesets that were found automatically"""
    # Open Github
    git = Github(token)
    # Get the repo and branch
    git_repo = git.get_repo(repo)
    git_branch = git_repo.get_branch(branch)
    head_sha = git_branch.commit.sha
    # Get the available files
    relative_path = os.getcwd()
    available_files = get_all_file_paths(relative_path, path=relative_path)
    # Create a list for the detected changes
    changed_files = set()
    for file, file_content in available_files:
        # Create the tree element
        changed_files.add(InputGitTreeElement(
            path=file,
            mode='100644',
            type='blob',
            content=file_content
            # sha=blob.sha
        ))
    # Get the base tree and the new tree
    base_tree = git_repo.get_git_tree(sha=head_sha)
    tree = git_repo.create_git_tree(changed_files, base_tree)
    # Apply the commit for the new bump
    bump_commit_message = "🔖 PR: Create Pull Request for new ChangeForge version"
    # Create the git commit applying the Changesets
    commit = git_repo.create_git_commit(
        message=bump_commit_message,
        tree=tree,
        parents=[git_repo.get_git_commit(head_sha)]
    )
    # Push the commit
    head_new_branch = git_repo.get_git_ref(ref=f'heads/{branch}')
    head_new_branch.edit(sha=commit.sha)


def open_pull_request(token: str, repo: str, branch: str) -> None:
    """Open the Github pull request"""
    # Open Github
    git = Github(token)
    # Get the repo and branch
    git_repo = git.get_repo(repo)
    # git_branch = git_repo.get_branch(branch)

    # Set some default values
    branch_pr = "bump-new-version"
    pr_body = "### 🚀 New Bump\nAutomatically bumping based on latest changesets.\n"
    pr_title = "Bump new project to version v"

    # Add the latest CHANGELOG perform
    useful_changelog_changes = "# Changelog\n"
    add_content: bool = False
    with open("CHANGELOG.md", "r", encoding="utf-8") as changelog:
        for line in changelog.readlines():
            if line.startswith("## ["):
                add_content = not add_content
                # Also, update the PR title adding the new
                pr_title += line.replace("## [", "").replace("]", "")
                # If you've add all the content on this new
                # bumped version, then break everything
                if add_content is False:
                    break
            # Add the content ONLY if the add content is True
            if add_content is True:
                useful_changelog_changes += line

    # Update the PR body using this changelog changes
    pr_body += useful_changelog_changes

    # Check if there's already an open PR for this branch
    existing_prs = git_repo.get_pulls(
        state="open", head=f"{git_repo.owner.login}:{branch_pr}"
    )
    if existing_prs.totalCount > 0:
        print("PR already exists for this branch")
        return

    # Create the Pull Request
    pr = git_repo.create_pull(title=pr_title, body=pr_body, head=branch_pr, base=branch)

    # Get CODEOWNERS and add them as reviewers
    try:
        codeowners = get_codeowners()
        if codeowners:
            pr.create_review_request(reviewers=codeowners)
            print(f"Added {len(codeowners)} CODEOWNERS as reviewers")
    except Exception as e:
        print(f"Failed to add CODEOWNERS as reviewers: {str(e)}")

    print("Pull request created")


if __name__ == "__main__":
    # Set the Github token
    github_token = os.getenv('GITHUB_TOKEN')
    # Get the repo and branch name
    REPO = os.getenv('REPO_NAME')
    BRANCH = os.getenv('BRANCH_NAME')
    # Call the Open Pull Request method
    open_pull_request(github_token, REPO, BRANCH)
