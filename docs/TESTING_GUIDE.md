# Testing the flow by hand

Copy-paste commands for driving the whole cycle yourself on Windows
(Scoop) and WSL (Homebrew), so you can show it rather than describe it.

Repo under test: <https://github.com/Chauhan-Mihir/homebrew-cx>

---

## First, what this does and does not prove

Two different "updates" are easy to conflate. Keep them apart when
presenting:

| | What it is | Tested here? |
| --- | --- | --- |
| **Package-manager update** | `scoop update cx`, `brew upgrade cx`. Driven by the manifests in this repo. | **Yes** |
| **In-binary update check** | `cx` itself noticing it is stale and printing a notice. Lives in `crates/update-check` in the toolkit, wired into `cx discover`. | **No** — the stand-in binary here has no update check |

The stand-in `cx` in this repo only prints its version. If you want the
demo to show the in-binary notice too, that has to be added to the
stand-in deliberately.

---

## Clean up first (Windows)

Your machine still has the artefacts of the earlier failed attempt: a
bucket pointing at the internal GHE host, and a `cx` entry marked
*Install failed*. Clear them so the demo starts from zero — and so there
is no ambiguity about which bucket a `cx` manifest came from.

```powershell
scoop uninstall cx        # ok if it errors; the earlier install never completed
scoop bucket rm cx        # the old github02.hclpnp.com bucket
scoop bucket list         # confirm it is gone
```

---

## Round 1 — install v1.0.0

### Windows / Scoop

```powershell
scoop bucket add cx https://github.com/Chauhan-Mihir/homebrew-cx
scoop install cx
cx --version
#   cx 1.0.0
```

### WSL / Homebrew

Homebrew is **not** currently installed in your Ubuntu WSL, so start there:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
```

Then:

```bash
brew tap Chauhan-Mihir/cx https://github.com/Chauhan-Mihir/homebrew-cx
brew install Chauhan-Mihir/cx/cx
cx --version
#   cx 1.0.0
```

Note the tap name is `Chauhan-Mihir/cx` while the repo is
`homebrew-cx` — Homebrew strips the `homebrew-` prefix by convention.

---

## Round 2 — publish v1.1.0

### The automated way (one command)

```bash
cd /c/ADO/cx-dist-rehearsal
git tag v1.1.0
git push origin v1.1.0
```

That tag triggers everything: build 4 targets, publish the release with
`SHA256SUMS`, regenerate all four manifests from those checksums, commit
them, tag `manifests-v1.1.0`, then run the install verification.

Watch it:

```bash
gh run watch --repo Chauhan-Mihir/homebrew-cx
```

### The manual way

See [MANUAL_RELEASE.md](MANUAL_RELEASE.md) — the same release done by
hand, which is the version worth walking the team through, because it
makes the ten hand-copied fields visible.

---

## Round 3 — the upgrade

This is the part that actually matters, and the part install tests say
nothing about.

### Windows / Scoop

```powershell
scoop update          # refreshes buckets (a git pull on each)
scoop status          # cx should be listed as having an update
scoop update cx
cx --version
#   cx 1.1.0
```

### WSL / Homebrew

```bash
brew update
brew outdated
brew upgrade Chauhan-Mihir/cx/cx
cx --version
#   cx 1.1.0
```

---

## Round 4 — what the maintainer scripts check

These two are the ones that fail outright against the internal GHE host,
because both fetch anonymously and that host answers `401`. Running them
against this repo is the clearest side-by-side evidence.

```powershell
$bin = "$env:USERPROFILE\scoop\apps\scoop\current\bin"
$dir = "$env:USERPROFILE\scoop\buckets\cx\bucket"

& "$bin\checkver.ps1"    -App cx -Dir $dir
#   cx: 1.1.0            <- discovered the published version

& "$bin\checkhashes.ps1" -App cx -Dir $dir
#   cx: OK               <- downloaded the real asset, hash matches
```

`scoop checkver` is **not** a scoop subcommand — it is a maintainer
script inside the scoop app, so it has to be invoked by path.

---

## Round 5 — clean uninstall

```powershell
scoop uninstall cx
scoop bucket rm cx
```

```bash
brew uninstall Chauhan-Mihir/cx/cx
brew untap Chauhan-Mihir/cx
```

---

## If something fails

| Symptom | Cause |
| --- | --- |
| `scoop install cx` picks the wrong manifest | More than one bucket provides `cx`. Qualify it: `scoop install cx/cx` |
| `brew install cx` says no formula | Use the full name `Chauhan-Mihir/cx/cx`, or `brew tap` first |
| Hash mismatch on install | The manifest and the release asset disagree — exactly the failure mode the generated manifests exist to prevent |
| macOS: "cannot be opened because the developer cannot be verified" | The binary is unsigned. Out of scope here, but real `cx` releases will need signing and notarization |
| `scoop update cx` says already current | `scoop update` (no args) first — the bucket is a git clone and needs pulling |
