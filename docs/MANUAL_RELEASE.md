# The manual release, step by step

This is the same release the `release` workflow does, written out as the
hand-operated version. It exists for two reasons: to show the team what
Homebrew and Scoop actually need from us, and to make visible how a tap
drifts out of step when each field is edited by hand.

Run it against this rehearsal repo, not the real one.

---

## Step 1 — Build each target

```sh
export CX_VERSION=1.0.0

cargo build --release --manifest-path dummy/Cargo.toml \
  --target x86_64-unknown-linux-gnu
cargo build --release --manifest-path dummy/Cargo.toml \
  --target aarch64-apple-darwin
cargo build --release --manifest-path dummy/Cargo.toml \
  --target x86_64-apple-darwin
cargo build --release --manifest-path dummy/Cargo.toml \
  --target x86_64-pc-windows-msvc
```

**First thing worth saying out loud to the room:** you cannot run all four
of these from one machine. Apple will not let you link a macOS binary
anywhere but macOS. On a Windows laptop you can produce the Windows
artifact and, with a cross toolchain, the Linux one — the two macOS
artifacts need a Mac or a macOS CI runner.

That is not a detail. It is why the current tap has **no arm64 macOS
artifact at all**, and why its single `url` ends up serving a Linux ELF to
Apple Silicon users.

## Step 2 — Archive, flat

```sh
V=1.0.0
mkdir -p dist stage

cp target/x86_64-unknown-linux-gnu/release/cx stage/
tar -czf dist/cx-v$V-linux-x86_64.tar.gz -C stage cx

cp target/aarch64-apple-darwin/release/cx stage/
tar -czf dist/cx-v$V-darwin-arm64.tar.gz -C stage cx

cp target/x86_64-apple-darwin/release/cx stage/
tar -czf dist/cx-v$V-darwin-x86_64.tar.gz -C stage cx

cp target/x86_64-pc-windows-msvc/release/cx.exe stage/
(cd stage && zip ../dist/cx-v$V-windows-x86_64.zip cx.exe)
```

Flat on purpose — one `cx` / `cx.exe` at the archive root. That is what
lets the formula say `bin.install "cx"` and the Scoop manifest say
`"bin": "cx.exe"` with no `extract_dir`.

## Step 3 — Checksums

```sh
(cd dist && sha256sum cx-v$V-*.tar.gz cx-v$V-*.zip > SHA256SUMS)
cat dist/SHA256SUMS
```

The `<hash>␣␣<filename>` format matters: Scoop's `autoupdate.hash.regex`
parses this file to find the hash for a new version.

> **For the real toolkit:** filter this file to Standard-tier lines only.
> Today's `C:\ADO\homebrew-cx\dist\*\SHA256SUMS` lists `cx-advisor-pro` and
> `cx-pbc` alongside the Standard tier, which discloses the filenames and
> hashes of restricted products the moment it is attached to a public
> release.

## Step 4 — Publish the release

```sh
gh release create v$V dist/* \
  --title "cx v$V" \
  --notes "Distribution rehearsal build."
```

## Step 5 — Hand-edit the Homebrew formula

Open `Formula/cx.rb` and change **seven** fields:

| Field | Count |
| --- | --- |
| `version` | 1 |
| `url` | 3 (darwin-arm64, darwin-x86_64, linux-x86_64) |
| `sha256` | 3, each matching its own url |

```ruby
version "1.0.0"

on_macos do
  on_arm do
    url ".../cx-v1.0.0-darwin-arm64.tar.gz"
    sha256 "<paste the darwin-arm64 line from SHA256SUMS>"
  end
  on_intel do
    url ".../cx-v1.0.0-darwin-x86_64.tar.gz"
    sha256 "<paste the darwin-x86_64 line>"
  end
end
```

## Step 6 — Hand-edit the Scoop manifest

`bucket/cx.json`, three more fields: `version`, `architecture.64bit.url`,
`architecture.64bit.hash`.

## Step 7 — Commit and tag the manifests

```sh
git add Formula bucket
git commit -m "chore: bump cx to v1.0.0"
git push
git tag manifests-v1.0.0 && git push origin manifests-v1.0.0
```

The `manifests-v<version>` tag is what lets a later upgrade test install
the *previous* version. The release tag cannot be used for that: it points
at the commit before the manifests were bumped.

---

## Ten fields, every release

That is **ten** values, copied by hand, that must agree with each other and
with the files actually attached to the release. Nothing checks them. A
wrong `sha256` is not caught until a user runs `brew install` and gets a
checksum mismatch; a wrong `url` is not caught until they get a 404.

This is not hypothetical. In `C:\ADO\homebrew-cx` today:

- `version` says `0.1.2-beta.261006130633`
- the `url` points at the `0.1.0-beta.261006130633` **Linux** tarball
- the `sha256` is the **Linux** hash
- and the formula has no platform branching, so macOS gets all three

Three different hosts appear across that repo's files
(`gh01.internal.company.com`, `github02.hclpnp.com`, `github.com/hcl-tech`),
which is the same failure in a different field.

## What the workflow does instead

`scripts/render_manifests.py` regenerates all four manifests from
`dist/SHA256SUMS`. Every url and hash is derived from the artifact that was
actually built and attached, in the same run. The ten hand-copied fields
become zero.

```
tag ──► build ×4 ──► SHA256SUMS ──► release ──► render manifests ──► verify ×4
```

The demo worth showing is the two side by side: run the manual steps, then
`git tag v1.1.0 && git push --tags` and watch the same thing happen with
one command and no opportunity to mistype a hash.
