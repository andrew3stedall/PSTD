# Repository Setup

## Known stack direction

User-provided expected stack areas:

- Rust
- Python
- React
- Vite
- Snowflake

## Current known repository state

The repository currently has minimal committed structure. The only confirmed root file before this planning scaffold was `README.md`.

## Unknown commands

These commands are not yet known and should not be invented by Codex:

- Test command
- Lint command
- Typecheck command
- Build command
- Data validation command

## Planning rule

When future issues are created, each issue should state the expected validation command if one exists. If no command exists yet, the issue should include setup work to add it.

## Recommended future setup issues

1. Establish Rust project structure.
2. Define PST extraction architecture.
3. Add a repeatable test command.
4. Add linting and formatting checks.
5. Add CI once commands are known.
6. Decide whether Python, React/Vite, and Snowflake are immediate requirements or later integrations.
