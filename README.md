<p align="center">
    <img src="https://github.com/ricardoleal20/ChangeForge/blob/main/docs/img/logo.png" width="50%" height="50%" />
</p>
<p align="center">
    <b>Tool for teams that manage the creation and modification of the </b>CHANGELOG<b> based on a specified set of changes.</b>
</p>

<!-- ![PyPi version](https://img.shields.io/pypi/v/changeforge?label=PyPi%20version&logo=PyPi&style=for-the-badge)
![Python versions supported](https://img.shields.io/pypi/pyversions/changeforge?label=Python%20Versions%20Supported&logo=Python&style=for-the-badge)
![Deployed](https://img.shields.io/github/actions/workflow/status/ricardoleal20/changeforge/.github/workflows/publish_on_release.yml?branch=main&label=LAST%20VERSION%20DEPLOYED%20%F0%9F%9A%80&logo=Github&style=for-the-badge)
![License](https://img.shields.io/github/license/ricardoleal20/changeforge?color=%23808000&label=%F0%9F%93%84%20LICENSE&style=for-the-badge) -->

## Installation

To install `ChangeForge`, you can do it through pip:

```
pip install changeforge
```

Please consider that it requires `Python >=3.9`

## Usage

This Rust package, `changeforge`, provides several command-line tools for managing project versions and changesets. Below are the available commands and their usage examples.

### Commands

#### `create`

Create a new changeset to document changes in the project.

```sh
changeforge create
```

This command creates a new changeset with the provided description. 

#### `list`

List all changesets created for the project.

```sh
changeforge list
```

This command displays a list of all changesets recorded in the project, along with their descriptions and types.

#### `bump`

Bump the project version according to the specified type.

```sh
changeforge bump
```

This command increments the project version based on the specified type: `major`, `minor`, or `patch`. It updates the version number in the project files accordingly.

Also, it deletes all the current `changesets` to avoid changes 

---

For more details on each command and its options, refer to the command-line help:

```sh
changeforge --help
```

## Contributing

Everyone can contribute. Before contributing, please read our [code of conduct](CODE_OF_CONDUCT.md).

To contribute to `ChangeForge`, follow these steps:

1. Fork this repository.
2. Create a new branch.
3. Make your changes and commit them.
4. Push your changes to your fork.
5. Create a pull request.

## License

Project Name is released under the [MIT License](LICENSE).

## Inspiration

Inspired by [Changesets](https://github.com/changesets/changesets).
