# Releasing

This project treats `v1.0.0` as the CLI contract freeze for the Mac-first
project mover. Current releases use GitHub Actions for validation and
tag-triggered Apple Silicon artifact generation.

## V1 Contract

- Commands: `plan`, `apply`, `verify`, `rollback`.
- Shared move flags: `--old`, `--new`, `--codex-home`, and
  `--allow-running-codex` where applicable.
- Apply-only flags: `--relink-only` and `--auto-rollback`.
- Rollback flag: `--backup`, accepting either a backup directory or the
  `manifest.json` file inside it.
- Rollback intentionally has no `--codex-home`; it restores absolute paths from
  the backup manifest.
- Human output is the default. Pass global `--json` before the subcommand for
  machine-readable output.
- Runtime failures use stable process exit codes. See `README.md` for the
  taxonomy.

## Release Checklist

1. Confirm the working tree is clean.
2. Confirm `README.md` documents Homebrew, source-build, and direct GitHub
   binary installation, including checksum verification and unsigned macOS
   quarantine handling.
3. Run:

   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   mkdir -p /tmp/codex-project-mover-release
   TMPDIR=/tmp/codex-project-mover-release cargo test --test git_worktree
   rustup toolchain install 1.85.0
   cargo +1.85.0 build --locked
   cargo build --release --locked
   ./target/release/codex-project-mover --version
   ```

4. Set the release version:

   ```bash
   VERSION=1.1.0
   TAG="v${VERSION}"
   ```

5. Update `Cargo.toml` to the release version, run
   `cargo check` so `Cargo.lock` records the new package version, then commit
   both files.
6. Create an annotated tag:

   ```bash
   git tag -a "$TAG" -m "Release $TAG"
   ```

7. Build the Apple Silicon macOS artifact:

   ```bash
   git describe --exact-match --tags HEAD
   test "$(uname -m)" = "arm64"
   cargo build --release --locked
   mkdir -p dist
   cp target/release/codex-project-mover dist/codex-project-mover-aarch64-apple-darwin
   file dist/codex-project-mover-aarch64-apple-darwin
   (cd dist && shasum -a 256 codex-project-mover-aarch64-apple-darwin > codex-project-mover-aarch64-apple-darwin.sha256)
   ```

8. Push the branch and tag, then wait for the release commit's CI run on
   `main` to pass before publishing the GitHub release.
9. Before using GitHub CLI release commands, run `gh auth status`. If `gh` is
   unavailable or unauthenticated, create the release from the GitHub web UI.
10. Create a GitHub release with the binary and checksum attached. Release notes
   must mention that the binary is unsigned and unnotarized.
11. For every release that updates the Homebrew formula, complete the Homebrew
    release gate in `docs/homebrew-packaging.md` before announcing the release
    as ready for normal users.

The `.github/workflows/release.yml` workflow runs on pushed `v*` tags and
uploads the Apple Silicon macOS binary plus SHA-256 checksum to the GitHub
release for that tag.

## Abort And Cleanup

If the release fails before anything is pushed, delete the local tag and either
fix forward or reset the local release commit:

```bash
git tag -d "$TAG"
git reset --hard HEAD~1
```

If the tag or release has already been pushed, remove the public release state
before retrying:

```bash
gh release delete "$TAG" --yes
git push origin ":refs/tags/$TAG"
git tag -d "$TAG"
git revert <release-commit>
```

If users may already have consumed the tag or artifact, do not rewrite it;
publish a fixed follow-up release instead.

## Unsigned macOS Binary

The v1 binary is unsigned and unnotarized. Users may need to remove the macOS
quarantine attribute after downloading from GitHub:

```bash
xattr -d com.apple.quarantine ./codex-project-mover-aarch64-apple-darwin
```

The release notes should mention this, along with the alternative Finder path:
right-click Open or System Settings > Privacy & Security > Open Anyway.

Post-v1 release improvements can add additional macOS architectures.

## Homebrew Tap

The project is available through the public `sunshineo/homebrew-tap` repository:

```bash
brew install sunshineo/tap/codex-project-mover
```

The tap formula currently supports Apple Silicon macOS and publishes one Sonoma
Apple Silicon bottle so Sonoma and newer Apple Silicon installs do not need to
build with Homebrew's Rust toolchain. Do not add Intel macOS, Linux, or older
macOS bottles until those platforms are verified and part of the supported CLI
contract.

See `docs/homebrew-packaging.md` for the tap model, bottle compatibility notes,
runner mapping, and verification commands.

The formula should point at immutable source tag tarballs such as
`https://github.com/sunshineo/codex-project-mover/archive/refs/tags/v1.1.0.tar.gz`.
Packaging-only tap updates do not require a new `codex-project-mover` release
version. Bump the CLI version only when source behavior, package metadata, or
release artifacts need a new upstream tag.
