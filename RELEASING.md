# Releasing kaya

The release workflow (`.github/workflows/release.yml`) runs on a `v*` tag and
publishes a five-target binary matrix, a GitHub Release, the Homebrew/Scoop
channels, and — when authorized — the crate to crates.io. Tagging and pushing
a tag are release actions: they need explicit human authorization for every
release, not a standing one.

## Prerequisites

- You are a maintainer with push access to `charly-vibes/kaya`.
- The `version` in `Cargo.toml` is the version you are about to release, and the
  working tree is clean.
- `cargo` 1.88 or newer locally.

## One-time: crates.io token

`cargo publish` authenticates with `CARGO_REGISTRY_TOKEN`, which the workflow
reads only inside the `publish` job. Without it the crates.io channel reports
`disabled` and the release still succeeds.

1. Create an API token at <https://crates.io/settings/tokens> with the
   `publish-new` and `publish-update` scopes, limited to the `kaya-vibes` crate.
2. Store it as a repository secret:

   ```sh
   gh secret set CARGO_REGISTRY_TOKEN --repo charly-vibes/kaya
   ```

   or add it under **Settings → Secrets and variables → Actions**.
3. Rotate it like any credential: create the new token, replace the secret, then
   revoke the old token on crates.io.

Once the secret exists, the next authorized tag publishes the crate and the
release summary reports `| crates | updated |` instead of `disabled`.

## Every release

1. Verify the crate tarball before tagging:

   ```sh
   cargo package --list --allow-dirty   # only src/, tests/ and repo metadata
   cargo publish --dry-run --allow-dirty
   ```

2. Check the tag matches the manifest version:

   ```sh
   grep '^version' Cargo.toml
   git tag "v$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
   ```

3. Push the tag (this is the authorized release action):

   ```sh
   git push origin "v<x.y.z>"
   ```

4. Watch the run: the `publish` job needs `test` and every build job. Its
   "Release channel propagation" table lists `crates`, `homebrew` and `scoop`
   as `updated`, `unchanged`, `disabled` or `failed`; any `failed` fails the run.

5. Confirm the result:

   ```sh
   cargo search kaya-vibes
   ```

   The release notes also link the crates.io version once the channel reports
   `updated`.

## If something goes wrong

- **Token absent or expired:** the channel reports `disabled`; replace the
  secret and re-run the failed `publish` job. Re-running is safe for the tap and
  bucket channels (they render byte-identical output) and for the crate (a
  version that already exists is rejected, never overwritten).
- **Wrong version published:** crates.io releases are immutable. Publish a
  corrected version, and `cargo yank --version <x.y.z>` the bad one so new
  resolutions avoid it while existing lockfiles keep working.
- **Tag pushed by mistake:** delete the tag only with maintainer agreement;
  GitHub Releases and crates.io versions already created are not rolled back by
  deleting the tag.