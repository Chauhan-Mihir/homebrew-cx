# Spike: do Homebrew and Scoop cope with a query string on the download URL?

## Why it matters

One candidate for hosting the real binaries is **Azure Blob Storage with a
read-only SAS token**. It is attractive because it satisfies the hard
constraint (the fetch is anonymous — the token rides in the URL, so no auth
header is needed and `brew`/`scoop` work unmodified) while still giving
back the control that a public GitHub release cannot:

- the container is not browsable or listable
- access is revocable — kill the SAS and every published URL dies at once
- Azure Storage logging records every download
- the token can be rotated per release

The one open question is mechanical: both package managers infer the
**download filename from the URL**, and a SAS URL ends in a long
`?sv=...&sig=...` query string. If that inference breaks, the option is
harder than it looks.

## How it is tested

`scripts/render_manifests.py` emits a second pair of manifests,
`Formula/cx-sas.rb` and `bucket/cx-sas.json`, identical to the real ones
except that every download URL carries a SAS-shaped query string. The
`sas-spike` job in `verify.yml` installs from them.

The query string is inert padding of the right shape — it is not a
credential, and nothing in this repo is actually protected by it:

```
?sv=2024-11-04&ss=b&srt=o&sp=r&se=2030-01-01T00:00:00Z
&st=2026-01-01T00:00:00Z&spr=https&sig=AAAA...%3D
```

This tests URL and filename *handling*, which is the actual risk. It does
not test SAS validity, which is not in doubt.

The job is `continue-on-error: true` on purpose — its value is the answer,
not a pass.

## Caveat on faithfulness

The spike appends the query string to a **GitHub release asset URL**, which
redirects to `objects.githubusercontent.com` carrying its own query
parameters. GitHub may ignore, strip, or reject the extra parameters, in
which case the simulation is not faithful and the result tells us nothing
about Azure.

Read the job log before trusting a green tick:

- **Install succeeds, and the log shows the archive cached under a sane
  filename** — the inference holds, Option A is clear on this point.
- **Install fails on a filename or extension error** — the inference
  breaks. Known workaround for Scoop: the `#/filename.zip` URL fragment
  forces a download name. Homebrew has no equally clean equivalent, which
  would count meaningfully against Option A.
- **GitHub rejected or stripped the query string** — inconclusive. Re-run
  against a real Azure Blob SAS, or any static host that passes arbitrary
  query strings through.

## Result

**Both package managers handle it. Verified on release v1.3.0
(run 38581353774).**

| Platform | Outcome | Evidence |
|---|---|---|
| Homebrew (Linux) | ✅ installed, reported `cx 1.3.0` | cache key below |
| Scoop (Windows) | ✅ installed, reported `cx 1.3.0` | job passed |

### Why this is not a false pass

The caveat above was real: had GitHub stripped the extra query parameters
before serving the file, the install would have succeeded while proving
nothing. It did not.

Homebrew names cached downloads `<sha256-of-the-URL>--<basename>`, and
the CI log shows:

```
2b36acad2a2a4c412425a8a55186923b729a4a329f4d48e80d9dbef0207698c0--cx-v1.3.0-linux-x86_64.tar.gz
```

Comparing that key against both forms of the URL:

```
sha256(url WITHOUT query)  64558e0687602b756984d2a2f772b228d96856af654f07d33d894c7f79fa6e65
sha256(url WITH SAS query) 2b36acad2a2a4c412425a8a55186923b729a4a329f4d48e80d9dbef0207698c0  <-- the cache key
```

The key matches the URL **with** the query string. Homebrew therefore
fetched the full SAS-style URL, and still derived the archive name as
`cx-v1.3.0-linux-x86_64.tar.gz` rather than something containing `?sv=`.
Filename inference is unaffected by a query string.

### Conclusion for the hosting memo

**Option A (Azure Blob + read-only SAS) is viable on this point.** The
caveat "needs a spike on Homebrew filename inference with a query string"
can be struck from the comparison table.

Two things this does **not** establish, which should stay on the list:

- **SAS expiry and rotation.** A SAS in a public manifest is valid until
  it expires. Rotating it per release means every older manifest's URL
  dies, so `scoop install cx@1.0.0` and `brew install cx@1.0.0` would stop
  working for pinned versions. Decide deliberately whether old versions
  must stay installable.
- **Azure's own behaviour.** This used a GitHub asset URL with
  SAS-shaped parameters appended. It proves the *package managers* cope
  with a query string; it does not exercise real SAS authentication,
  Azure's error responses, or its redirect behaviour.
