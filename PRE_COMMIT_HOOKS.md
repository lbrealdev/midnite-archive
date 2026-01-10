## Pre-commit Hooks

This project uses [pre-commit](https://pre-commit.com/) to maintain code quality and consistency. Pre-commit hooks run automatically before each commit to catch issues early.

### Setup

Pre-commit is already configured in `.pre-commit-config.yaml`. To set it up:

```shell
# Install pre-commit (if not already installed)
pip install pre-commit

# Install the hooks
pre-commit install

# Optional: install hooks that run on push
pre-commit install --hook-type pre-push
```

### Available Hooks

- **trailing-whitespace**: Removes trailing whitespace from `.sh` and `.md` files
- **end-of-file-fixer**: Ensures files end with a newline
- **check-executables-have-shebangs**: Verifies executable scripts have shebangs
- **check-shebang-scripts-are-executable**: Ensures scripts with shebangs are executable
- **shellcheck**: Lints bash scripts for common issues and style problems
- **commitizen**: Validates commit messages follow conventional commit format
- **General checks**: YAML, JSON, TOML validation, merge conflict detection

### Usage

```shell
# Run all hooks on all files
pre-commit run --all-files

# Run specific hook
pre-commit run shellcheck --all-files

# Run hooks on staged files only
pre-commit run

# Update hook versions
pre-commit autoupdate

# Bypass hooks (use sparingly)
git commit --no-verify
```

### CI Integration

Pre-commit is configured for CI with automatic updates and auto fixing enabled. The configuration includes:

- Weekly automatic updates of hook versions
- Automatic fixing of fixable issues
- CI skipping for certain commit messages

### Customization

To modify hooks, edit `.pre-commit-config.yaml`. Common customizations:

- Add new file patterns to existing hooks
- Enable/disable specific hooks
- Add new repositories with additional hooks
- Adjust hook arguments

See the [pre-commit documentation](https://pre-commit.com/) for more options.
