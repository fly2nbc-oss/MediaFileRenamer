# Contributing

Thank you for helping improve **Media File Renamer**.

## Getting started

1. Fork the repository and create a branch from `main` (or `master`).
2. Install prerequisites listed in the **Development & Build** section of [`README.md`](./README.md).
3. Run `npm install` and `npm run tauri dev` for local development.

## Pull requests

- Keep changes focused on one concern where possible.
- Describe **what** changed and **why** in the PR description.
- Ensure `npm run build` and `cargo check` (under `src-tauri/`) succeed before submitting.

## Versioning

Release versioning is automated via `npm run version:patch`, `version:minor`, `version:major`, or `version:auto`. See [`docs/versioning.md`](./docs/versioning.md).

## Code of conduct

Participants are expected to follow our [`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md).
