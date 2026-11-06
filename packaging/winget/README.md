# Publishing gash to winget

The winget package is `hex2f.gash`. Each release ships `gash-<version>-x64.zip` and
`gash-<version>-arm64.zip` (gash.exe, LICENSE, README.md), and winget installs the zip as a
portable app: it unpacks into `%LOCALAPPDATA%\Microsoft\WinGet\Packages\hex2f.gash_...` and
puts `gash` on PATH (a symlink in `WinGet\Links` with Developer Mode, otherwise that folder is
added to the user PATH). The package depends on `Git.Git`, since gash runs Git for Windows'
tools; winget installs Git only when it doesn't find it already installed.

`.github/workflows/release.yml` runs when a `v*` tag is pushed. It builds both architectures
(checking that the tag matches `Cargo.toml` and that the exe doesn't need the VC++ runtime),
publishes the GitHub release with the zips and `SHA256SUMS.txt`, and then submits the new
version to winget once the `WINGET_TOKEN` secret exists.

## First version (by hand)

winget's automation can only update a package that is already in
[microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs), so 0.1.0 is submitted with
the manifests in `manifests/h/hex2f/gash/0.1.0/` (the same layout as winget-pkgs).

1. Tag and push: `git tag v0.1.0`, then `git push origin v0.1.0`, and wait for the Release
   workflow to publish the release.
2. In `hex2f.gash.installer.yaml`, replace the two `SHA256-OF-...` placeholders with the
   hashes from the release's `SHA256SUMS.txt`, and `RELEASE-DATE` with the release date
   (`YYYY-MM-DD`).
3. Check it: `winget validate --manifest packaging/winget/manifests/h/hex2f/gash/0.1.0`.
   To also test-install it locally, an administrator has to run
   `winget settings --enable LocalManifestFiles` first; then
   `winget install --manifest packaging/winget/manifests/h/hex2f/gash/0.1.0`.
4. Fork microsoft/winget-pkgs to the `hex2f` account, copy the `0.1.0` folder to
   `manifests/h/hex2f/gash/0.1.0/` in the fork, and open a pull request. (Or let
   `wingetcreate submit --token <PAT> <folder>` do the fork and PR.)

The PR goes through winget's automated validation and then a moderator's review; a new
package usually takes a few days.

## Later versions (automatic)

After the first version is merged:

1. Create a *classic* GitHub personal access token with the `public_repo` and `workflow`
   scopes (fine-grained tokens aren't supported by the action).
2. Save it as the repository secret `WINGET_TOKEN`: `gh secret set WINGET_TOKEN -R hex2f/gash`.
3. For each release, bump `version` in `Cargo.toml`, run `cargo build` so `Cargo.lock`
   follows, commit, then tag `vX.Y.Z` and push the tag.

The `winget` job then uses [winget-releaser](https://github.com/vedantmgoyal9/winget-releaser)
to open the PR on microsoft/winget-pkgs from the `hex2f` fork, based on the latest version
already there (so later versions don't use the manifests in this folder). The action is
pinned to a commit; update the hash in the workflow to pick up fixes.
