# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0]

### Feature

- src/utilities/ai_calls, src/utilities/ai_message_generator.rs, src/utilities/version_operations.rs: Add AI-powered changeset message generation with support for OpenAI and Gemini providers. Implement environment-based configuration through single API_KEY and MODEL variables. Include automatic version calculation based on semantic versioning rules..
- Bump New Version: Based on the different paths for finding the version, bump the version on all of those files at the same time.

### PATCH

- src/options/create.rs: Change the CHANGESET to include the version as one of the parameters usable for it..
- Build with Pyproject: Add the Maturin BackEnd for the build process, and include a workflow for the automatic publishing of the repo once we create new tags.

### Add

- VersionWise backend in Rust: Write the VersionWise BackEnd with Rust, that includes the methods `create` to create changesets, `bump` to use the current changesets to update the version and the changelog and `list` to list the current changes.
- Package: Include documentation and help methods for all the method of the VersionWise CLI tool.
- GitHub Workflows: Add a GitHub workflow for automatic Pull Requests opening and automatic updates.
- AI-powered changeset message generation with support for multiple providers (OpenAI and Gemini)
- Environment-based configuration for AI providers through `.env` file
- Automatic version calculation based on change type (MAJOR, MINOR, PATCH)

### Changed
- Improved changeset message formatting for better clarity and consistency
- Enhanced version management system with semantic versioning support

### Technical
- Added new modules for AI integration (`ai_calls`, `version_operations`)
- Implemented proper error handling for AI API calls
- Added message cleanup to ensure consistent formatting
