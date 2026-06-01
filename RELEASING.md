# Releasing

This project treats `v1.0.0` as the CLI contract freeze for the Mac-first
project mover. The release process is intentionally manual until release
automation exists.

## V1 Contract

- Commands: `plan`, `apply`, `verify`, `rollback`.
- Shared move flags: `--old`, `--new`, `--codex-home`, and
  `--allow-running-codex` where applicable.
- Apply-only flag: `--relink-only`.
- Rollback flag: `--backup`, accepting either a backup directory or the
  `manifest.json` file inside it.
- Rollback intentionally has no `--codex-home`; it restores absolute paths from
  the backup manifest.
- Output is human-oriented. Scripts should rely on process exit status for v1.

## Release Checklist

1. Confirm the working tree is clean.
2. Run:

   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   cargo build --release
   ./target/release/codex-project-mover --version
   ```

3. Update `Cargo.toml` to the release version, such as `1.0.0`, and commit it.
4. Create an annotated tag:

   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   ```

5. Build the Apple Silicon macOS artifact:

   ```bash
   cargo build --release
   mkdir -p dist
   cp target/release/codex-project-mover dist/codex-project-mover-aarch64-apple-darwin
   shasum -a 256 dist/codex-project-mover-aarch64-apple-darwin > dist/codex-project-mover-aarch64-apple-darwin.sha256
   ```

6. Push the branch and tag, then create a GitHub release with the binary and
   checksum attached.

Post-v1 release improvements can add GitHub Actions release automation,
additional macOS architectures, Homebrew packaging, and machine-readable output.
