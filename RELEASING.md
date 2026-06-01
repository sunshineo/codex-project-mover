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
2. Confirm `README.md` documents source-build and direct GitHub binary
   installation, including checksum verification and unsigned macOS quarantine
   handling. Homebrew should remain documented only as post-v1.
3. Run:

   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   mkdir -p /tmp/codex-project-mover-release
   TMPDIR=/tmp/codex-project-mover-release cargo test --test git_worktree
   rustup toolchain install 1.78.0
   cargo +1.78.0 build --locked
   cargo build --release --locked
   ./target/release/codex-project-mover --version
   ```

4. Update `Cargo.toml` to the release version, such as `1.0.0`, run
   `cargo check` so `Cargo.lock` records the new package version, then commit
   both files.
5. Create an annotated tag:

   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   ```

6. Build the Apple Silicon macOS artifact:

   ```bash
   git describe --exact-match --tags HEAD
   test "$(uname -m)" = "arm64"
   cargo build --release --locked
   mkdir -p dist
   cp target/release/codex-project-mover dist/codex-project-mover-aarch64-apple-darwin
   file dist/codex-project-mover-aarch64-apple-darwin
   (cd dist && shasum -a 256 codex-project-mover-aarch64-apple-darwin > codex-project-mover-aarch64-apple-darwin.sha256)
   ```

7. Push the branch and tag, then wait for the release commit's CI run on
   `main` to pass before publishing the GitHub release.
8. Before using GitHub CLI release commands, run `gh auth status`. If `gh` is
   unavailable or unauthenticated, create the release from the GitHub web UI.
9. Create a GitHub release with the binary and checksum attached. Release notes
   must mention that the binary is unsigned and unnotarized.

## Abort And Cleanup

If the release fails before anything is pushed, delete the local tag and either
fix forward or reset the local release commit:

```bash
git tag -d v1.0.0
git reset --hard HEAD~1
```

If the tag or release has already been pushed, remove the public release state
before retrying:

```bash
gh release delete v1.0.0 --yes
git push origin :refs/tags/v1.0.0
git tag -d v1.0.0
git revert <release-commit>
```

If users may already have consumed the tag or artifact, do not rewrite `v1.0.0`;
publish a fixed follow-up release instead.

## Unsigned macOS Binary

The v1 binary is unsigned and unnotarized. Users may need to remove the macOS
quarantine attribute after downloading from GitHub:

```bash
xattr -d com.apple.quarantine ./codex-project-mover-aarch64-apple-darwin
```

The release notes should mention this, along with the alternative Finder path:
right-click Open or System Settings > Privacy & Security > Open Anyway.

Post-v1 release improvements can add GitHub Actions release automation,
additional macOS architectures, Homebrew packaging, and machine-readable output.
The Homebrew formula can be published after v1.0.0 using GitHub's immutable tag
tarball at `https://github.com/sunshineo/codex-project-mover/archive/refs/tags/v1.0.0.tar.gz`.
