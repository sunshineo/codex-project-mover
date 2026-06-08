# Homebrew Packaging Notes

This note records the packaging decisions behind `sunshineo/tap/codex-project-mover`.
It is meant to keep the Homebrew context out of chat history and make future tap
maintenance less surprising.

## Current Policy

- The upstream CLI version remains `1.1.0` for Homebrew-only packaging changes
  after the `v1.1.0` release is published.
- The tap formula lives in the public `sunshineo/homebrew-tap` repository.
- The formula supports Apple Silicon macOS only:
  - `depends_on arch: :arm64`
  - `depends_on :macos`
- The tap publishes one bottle for `arm64_sonoma`, built on GitHub's `macos-14`
  Apple Silicon runner.
- Do not keep separate `arm64_tahoe` or `arm64_sequoia` bottles unless a real
  compatibility problem appears.
- Do not add Intel macOS, Linux, or older macOS bottles until those platforms are
  verified and intentionally supported by the CLI.

The goal is a small, easy-to-maintain tap: one oldest-supported Apple Silicon
macOS bottle, usable by newer Apple Silicon macOS releases through Homebrew's
older-compatible bottle selection.

## Tap Model

A Homebrew tap is just a Git repository containing formula files. A personal tap
such as `sunshineo/homebrew-tap` does not need Homebrew/core approval. Users opt
into it explicitly:

```bash
brew install sunshineo/tap/codex-project-mover
```

Homebrew/core is different. Submitting there involves Homebrew maintainers and
their review rules. This project currently uses a personal tap, so the main
things to protect are formula correctness, checksum immutability, release asset
availability, and a respectful install experience.

## Bottles

A bottle is Homebrew's prebuilt binary package for a formula. Without a matching
bottle, Homebrew builds from source. For this Rust CLI, a source build means
users may download Homebrew's Rust build dependency and its large dependency
chain. During local verification, that path installed large packages such as
Rust and LLVM, while the installed CLI itself was only a few megabytes.

The bottle avoids that cost. A normal bottle install downloads the small bottle
tarball and does not need Homebrew's Rust toolchain at install time.

The formula's bottle block records:

- `root_url`: where bottle assets are hosted.
- `rebuild`: bottle rebuild number for the same upstream version.
- `sha256`: the checksum for each OS/architecture bottle tag.
- `cellar: :any_skip_relocation`: the bottle does not need relocation work for a
  specific Cellar path.

Official reference: [Homebrew Bottles](https://docs.brew.sh/Bottles).

## GitHub Runners And Bottle Tags

GitHub provides the machine used to build the bottle. Homebrew names the bottle
tag from the OS and CPU architecture detected during the build.

The names are related, but not textually identical:

| GitHub runner | macOS release | Homebrew bottle tag |
| --- | --- | --- |
| `macos-14` | Sonoma | `arm64_sonoma` |
| `macos-15` | Sequoia | `arm64_sequoia` |
| `macos-26` | Tahoe | `arm64_tahoe` |

Official GitHub runner references:

- [GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/github-hosted-runners-reference)
- [actions/runner-images](https://github.com/actions/runner-images)

Runner availability changes over time. As of 2026-06-01, `macos-14` is
available for Apple Silicon builds, but GitHub's runner image release notes say
the macOS 14 image begins deprecation on 2026-07-06 and becomes unsupported on
2026-11-02. Re-check the official runner docs before changing the minimum
bottle target.

## Compatibility Direction

The practical compatibility direction is old-to-new:

- A bottle built on older macOS is generally usable on newer macOS of the same
  architecture.
- A bottle built on newer macOS should not be assumed usable on older macOS.

Homebrew's macOS bottle selection also falls back to an older compatible bottle
tag for the same architecture when an exact tag is not present. In local
Homebrew source, this behavior is implemented by
`find_older_compatible_tag` in:

```text
/opt/homebrew/Library/Homebrew/extend/os/mac/utils/bottles.rb
```

That is why the tap builds `arm64_sonoma` instead of `arm64_tahoe`: Sonoma is the
oldest Apple Silicon macOS runner currently targeted, and newer Apple Silicon
machines can use that bottle.

This is not a universal guarantee for every program. A binary can still depend
on APIs or linked libraries that are unavailable on some OS version. For this
project, the risk is low because it is a small Rust CLI with no Homebrew runtime
dependencies, but the right evidence is still a real install and workflow test.

## Current Workflow

The tap's `brew test-bot` workflow uses:

```yaml
os: [ macos-14 ]
```

The formula build step explicitly targets the formula:

```bash
brew test-bot --only-formulae codex-project-mover
```

Do not use `--keep-old` for the current policy. `--keep-old` preserves existing
bottle tags and is useful when intentionally accumulating multiple bottle tags,
but the current decision is Sonoma-only.

## Verification To Run

For a packaging-only tap change, useful verification is:

```bash
brew style --formula sunshineo/tap/codex-project-mover
brew audit --formula sunshineo/tap/codex-project-mover --skip-style
brew reinstall sunshineo/tap/codex-project-mover
brew test sunshineo/tap/codex-project-mover
```

On a newer Apple Silicon macOS machine, confirm the reinstall pours the Sonoma
bottle:

```text
Pouring codex-project-mover-1.1.0.arm64_sonoma.bottle.1.tar.gz
```

Then inspect the install receipt:

```bash
jq '{built_as_bottle, poured_from_bottle, runtime_dependencies, built_on}' \
  /opt/homebrew/Cellar/codex-project-mover/1.1.0/INSTALL_RECEIPT.json
```

Expected high-level result:

- `built_as_bottle: true`
- `poured_from_bottle: true`
- `runtime_dependencies: []`
- `built_on.os_version` shows macOS 14.x for the Sonoma bottle

`brew test` is only a smoke test. It proves the CLI starts and prints expected
help/version output. For stronger confidence, run a fake-project move with a
temporary Codex home and `CODEX_PROJECT_MOVER_TEST_TRASH_DIR`, then run
`verify`. This exercises the actual packaged binary without touching real Codex
projects.

## Release Gate For New Versions

Use this gate for every upstream release that should be installable through
Homebrew. The intent is to prove the release works on the oldest supported
Apple Silicon runner before treating the release as ready.

The Homebrew formula must point at an immutable source tag, so the source tag
needs to exist before the tap can build the bottle. Treat the release as
unfinished until this gate passes.

1. Finish the upstream release commit and version bump.
2. Run the normal project release checks from `RELEASING.md`.
3. Push the release branch and tag so the immutable source tarball exists.
4. Confirm the upstream project CI passes on the oldest supported Apple Silicon
   runner. Today that is `macos-14`.
5. Update the tap formula to the new tag tarball and source SHA-256.
6. Keep the bottle policy Sonoma-only unless the support policy changes:
   - tap workflow runner: `macos-14`
   - bottle tag: `arm64_sonoma`
   - no `--keep-old` in the bottle build command
7. Open the tap PR and wait for `brew test-bot` to build and test the formula
   on the oldest supported runner.
8. If the `macos-14` build fails, stop the release. Do not fall back to a newer
   runner just because Tahoe works locally. Either fix Sonoma support, formally
   change the minimum supported macOS, or use a self-hosted runner for the older
   target.
9. Publish the bottle through the tap workflow.
10. On a newer local Apple Silicon Mac, reinstall from Homebrew and confirm it
    pours the oldest-runner bottle rather than source-building:

    ```bash
    brew update
    brew reinstall sunshineo/tap/codex-project-mover
    ```

    Expected output includes:

    ```text
    Pouring codex-project-mover-<version>.arm64_sonoma.bottle.<rebuild>.tar.gz
    ```

11. Inspect the install receipt:

    ```bash
    jq '{built_as_bottle, poured_from_bottle, runtime_dependencies, built_on}' \
      /opt/homebrew/Cellar/codex-project-mover/<version>/INSTALL_RECEIPT.json
    ```

    Expected result:

    - `built_as_bottle: true`
    - `poured_from_bottle: true`
    - `runtime_dependencies: []`
    - `built_on.os_version` is macOS 14.x for the current Sonoma policy

12. Run the Homebrew smoke test:

    ```bash
    brew test sunshineo/tap/codex-project-mover
    ```

13. Run a packaged-binary fake move test. This exercises `apply` and `verify`
    without touching real Codex projects:

    ```bash
    set -euo pipefail

    root=$(mktemp -d /tmp/codex-project-mover-bottle-e2e.XXXXXX)
    home="$root/.codex"
    old="$root/old-project"
    new="$root/nested/new-project"
    trash="$root/trash"

    mkdir -p "$home/sessions" "$old/src"
    printf 'fn main() {}\n' > "$old/src/main.rs"
    printf '{"cwd":"%s"}\n' "$old" > "$home/sessions/thread.jsonl"

    CODEX_PROJECT_MOVER_TEST_TRASH_DIR="$trash" \
      /opt/homebrew/bin/codex-project-mover apply \
        --old "$old" \
        --new "$new" \
        --codex-home "$home" \
        --allow-running-codex

    /opt/homebrew/bin/codex-project-mover verify \
      --old "$old" \
      --new "$new" \
      --codex-home "$home" \
      --allow-running-codex

    test -f "$new/src/main.rs"
    test ! -e "$old"
    test -d "$trash/old-project"
    rg -q "$new" "$home/sessions/thread.jsonl"
    ! rg -q "$old" "$home/sessions/thread.jsonl"
    ```

14. Confirm the release asset list matches the support policy. For the current
    policy, the Homebrew release should contain one bottle asset:

    ```text
    codex-project-mover-<version>.arm64_sonoma.bottle.<rebuild>.tar.gz
    ```

Only after this gate passes should the release be announced as ready for normal
Homebrew users.

If a failure happens before users have consumed the tag or release, remove the
public release state and retry. If users may already have consumed it, do not
rewrite history; publish a fixed follow-up release.

## Release Versioning

Publishing or changing Homebrew bottle metadata does not require a new upstream
`codex-project-mover` version. Keep the formula pointed at the immutable
`v1.1.0` source tarball unless source behavior, package metadata, or upstream
release artifacts need a new project release.

Use a new upstream version when the CLI itself changes. Use bottle rebuilds and
tap commits for packaging-only corrections.
