# cx distribution rehearsal

A throwaway Homebrew tap and Scoop bucket used to prove the whole
distribution path — build, release, manifest, install, **upgrade**,
uninstall — before any of it is pointed at real `cx` binaries.

The binary this repo publishes is a stand-in: a ~100-line Rust program that
prints its version and exits. It contains no product logic, no network
access and no dependencies, so there is nothing proprietary here.

## Why this exists

The real tap could not be validated, for a reason that is structural rather
than a misconfiguration:

- Homebrew and Scoop download from the **end user's machine**, with a plain
  unauthenticated HTTP GET. Neither has a supported place to put a
  credential.
- The internal GitHub Enterprise host answers `/api/v3/...` with
  `401 Must authenticate to access this API`, so anonymous fetches — from
  `brew`, from `scoop`, and from the CLI's own update check — all fail.

So the download URL has to be anonymously reachable. This repo establishes
what "working" looks like against a host that is, so the real cleanup is a
find-and-replace rather than a redesign.

## Layout

```
dummy/                      tiny Rust crate; [[bin]] name = "cx"
scripts/render_manifests.py regenerates all four manifests from SHA256SUMS
Formula/cx.rb               Homebrew formula          (generated)
Formula/cx-sas.rb           query-string spike        (generated)
bucket/cx.json              Scoop manifest            (generated)
bucket/cx-sas.json          query-string spike        (generated)
.github/workflows/release.yml   build -> release -> bump manifests -> verify
.github/workflows/verify.yml    install / upgrade / uninstall on 4 platforms
```

## Setup

The repository **must** be named `homebrew-cx` and be **public** — Homebrew
derives the tap name `<owner>/cx` from the `homebrew-` prefix, and both
package managers fetch anonymously.

```sh
git remote add origin https://github.com/<owner>/homebrew-cx.git
git push -u origin main
```

Nothing else needs configuring. The workflows read `GITHUB_REPOSITORY` and
use the built-in `GITHUB_TOKEN`, so the owner is picked up automatically and
no secrets are required.

## Cutting a release

```sh
git tag v1.0.0 && git push origin v1.0.0
```

That one command runs the full chain:

| Stage | What it does |
|---|---|
| `build` | Builds 4 targets on real runners, archives each one |
| `release` | Generates `SHA256SUMS`, creates the GitHub Release |
| `bump` | Regenerates all four manifests, commits, tags `manifests-v1.0.0` |
| `verify` | Installs on macOS arm64, macOS x86_64, Linux and Windows |

## Testing an upgrade

Install tests alone prove nothing about updates, so the upgrade path is
tested explicitly. Release a second version, then run the **verify**
workflow manually with `upgrade_from` set:

```
expect_version: 1.1.0
upgrade_from:   1.0.0
```

Each job then installs 1.0.0 by checking the tap out at its
`manifests-v1.0.0` tag, asserts it, returns to `main`, upgrades, and asserts
1.1.0. The release tag itself cannot be used for this: it points at the
commit *before* the manifest bump, which is why `bump` writes a separate
`manifests-v<version>` tag.

## Artifact naming

```
cx-v<version>-<os>-<arch>.<ext>

cx-v1.0.0-darwin-arm64.tar.gz
cx-v1.0.0-darwin-x86_64.tar.gz
cx-v1.0.0-linux-x86_64.tar.gz
cx-v1.0.0-windows-x86_64.zip
SHA256SUMS
```

This deliberately differs from the current real convention
(`cx-advisor-v<version>-darwin.tar.gz`), which carries the OS but **no
arch**. That omission is why the real formula cannot express Apple Silicon
and currently serves a Linux ELF to macOS. The arch-bearing name is what
makes the `on_arm` / `on_intel` branching expressible, and the real pipeline
needs to adopt it.

Archives stay flat — a single `cx` / `cx.exe` at the root — so
`bin.install "cx"` and `"bin": "cx.exe"` work without `extract_dir`. That
part matches the real artifacts already.

## Manifests are generated, never hand-edited

`scripts/render_manifests.py` rewrites all four files from the release's
`SHA256SUMS`. The real tap drifted into a state where `version`, `url` and
`sha256` disagreed with one another — which is only reachable when each
field is edited by hand. Generating them makes that class of error
impossible.

## What this does not cover

- **Code signing and notarization.** An unsigned binary from the internet
  hits Gatekeeper on macOS and SmartScreen on Windows. Real `cx` releases
  will need signing; this rehearsal says nothing about it.
- **Whether the real binaries should be published publicly at all.** That
  is the open hosting question. This repo only establishes that the
  mechanism works when the URL is anonymously reachable.
