# Release Process

This is intended to serve as documentation for Wasmtime's release process. It's
largely an internal checklist for those of us performing a Wasmtime release, but
others might be curious in this as well!

## Releasing a major version

Major versions of Wasmtime are relased once-a-month. Most of this is automatic
and all that needs to be done is to merge GitHub PRs that CI will
generate. At a high-level the structure of Wasmtime's release process is:

* On the 5th of every month a new `release-X.Y.Z` branch is created with the
  current contents of `main`.
* On the 20th of every month this release branch is published to crates.io and
  release artifacts are built.

This means that Wasmtime releases are always at least two weeks behind
development on `main` and additionally happen once a month. The lag time behind
`main` is intended to give time to fuzz changes on `main` as well as allow
testing for any users using `main`. It's expected, though, that most consumers
will likely use the release branches of wasmtime.

A detailed list of all the steps in the release automation process are below.
The steps requiring interactions are **bolded**, otherwise everything else is
automatic and this is documenting what automation does.

1. On the 5th of every month, (configured via
   `.github/workflows/release-process.yml`) a CI job
   will run and do these steps:
   * Download the current `main` branch
   * Push the `main` branch to `release-X.Y.Z`
   * Run `./scripts/publish.rs` with the `bump` argument
   * Commit the changes
   * Push these changes to a temporary `ci/*` branch
   * Open a PR with this branch against `main`
   * This step can also be [triggered manually][ci-trigger] with the `main`
     branch and the `cut` argument.
2. **A maintainer of Wasmtime merges this PR**
   * It's intended that this PR can be immediately merged as the release branch
     has been created and all it's doing is bumping the version.
3. **Time passes and the `release-X.Y.Z` branch is maintained**
   * All changes land on `main` first, then are backported to `release-X.Y.Z` as
     necessary.
   * Even changes to `RELEASES.md` are pushed to `main` first.
4. On the 20th of every month (same CI job as before) another CI job will run
   performing:
   * Download the current `main` branch.
   * Update the release date of `X.Y.Z` to today in `RELEASES.md`
   * Open a PR against `main` for this change
   * Reset to `release-X.Y.Z`
   * Update the release date of `X.Y.Z` to today in `RELEASES.md`
   * Add a special marker to the commit message to indicate a tag should be made.
   * Open a PR against `release-X.Y.Z` for this change
   * This step can also be [triggered manually][ci-trigger] with the `main`
     branch and the `release-latest` argument.
5. **A maintainer of Wasmtime merges these two PRs**
   * The PR against `main` is a small update to the release notes and should be
     mergeable immediately.
   * The PR against `release-X.Y.Z`, when merged, will trigger the next steps due
     to the marker in the commit message. A maintainer should double-check there
     are [no open security issues][rustsec-issues], but otherwise it's expected
     that all other release issues are resolved by this point.
6. The `.github/workflow/push-tag.yml` workflow is triggered on all commits
   including the one just created with a PR merge. This workflow will:
   * Scan the git logs of pushed changes for the special marker added by
     `release-process.yml`.
   * If found, tags the current `main` commit and pushes that to the main
     repository.
7. Once a tag is created CI runs in full on the tag itself. CI for tags will
   create a GitHub release with release artifacts and it will also publish
   crates to crates.io. This is orchestrated by `.github/workflows/main.yml`.

If all goes well you won't have to read up much on this and after hitting the
Big Green Button for the automatically created PRs everything will merrily
carry on its way.

[rustsec-issues]: https://github.com/bytecodealliance/wasmtime/issues?q=RUSTSEC+is%3Aissue+is%3Aopen+
[ci-trigger]: https://github.com/bytecodealliance/wasmtime/actions/workflows/release-process.yml

## Releasing a patch version

Making a patch release is somewhat more manual than a major version, but like
before there's automation to help guide the process as well and take care of
more mundane bits.

This is a list of steps taken to perform a patch release for 2.0.1 for example.
Like above human interaction is indicated with **bold** text in these steps.

1. **Necessary changes are backported to the `release-2.0.0` branch from
   `main`**
   * All changes must land on `main` first (if applicable) and then get
     backported to an older branch. Release branches should already exist from
     the above major release steps.
   * CI may not have been run in some time for release branches so it may be
     necessary to backport CI fixes and updates from `main` as well.
   * When merging backports maintainers need to double-check that the
     `PUBLIC_CRATES` listed in `scripts/publish.rs` do not have
     semver-API-breaking changes (in the strictest sense). All security fixes
     must be done in such a way that the API doesn't break between the patch
     version and the original version.
   * Don't forget to write patch notes in `RELEASES.md` for backported changes.
2. **The patch release process is [triggered manually][ci-trigger] with
   the `release-2.0.0` branch and the `release-patch` argument**
   * This will run the `release-process.yml` workflow. The `scripts/publish.rs`
     script will be run with the `bump-patch` argument.
   * The changes will be committed with a special marker indicating a release
     needs to be made.
   * A PR will be created from a temporary `ci/*` branch to the `release-2.0.0`
     branch which, when merged, will trigger the release process.
3. **Review the generated PR and merge it**
   * This will resume from step 6 above in the major release process where the
     special marker in the commit message generated by CI will trigger a tag to
     get pushed which will further trigger the rest of the release process.

After a patch release has been made you'll also want to double-check that the
release notes on the patch branch are in sync with the `main` branch.

[bump-version]: https://github.com/bytecodealliance/wasmtime/actions/workflows/bump-version.yml

## Releasing a security patch

When making a patch release that has a security-related fix the contents of the
patch are often kept private until the day of the patch release which means that
the process here is slightly different from the patch release process above. In
addition the precise [runbook is currently under discussion in an
RFC](https://github.com/bytecodealliance/rfcs/pull/20) for security patches, so
this intends to document what we've been doing so far and it'll get updated when
the runbook is merged.

1. **The fix for the security issue is developed in a GitHub Security
   Advisory**
   * This will not have any CI run, it's recommended to run `./ci/run-tests.sh`
     locally at least.
   * This will also only be the patch for the `main` branch. You'll need to
     locally maintain and develop patches for any older releases being backported
     to. Note that from the major release process there should already be a
     branch for all older releases.
2. **Send a PR for the version bump when an email goes out announcing there will
   be a security release**
   * An email is sent to the bytecodealliance security mailing list ahead of a
     patch release to announce that a patch release will happen. At this time you
     should [trigger the version bump][ci-trigger] against the appropriate
     `release-x.y.z` branch with the `release-patch` argument.
   * This will send a PR, but you should not merge it. Instead use this PR and
     the time ahead of the security release to fix any issues with CI. Older
     `release-x.y.z` branches haven't run CI in awhile so they may need to
     backport fixes of one variety or another. DO NOT include the actual fix for
     the security issue into the PR, that comes in the next step.
3. **Make the patches public**
   * For the `main` branch this will involve simply publishing the GitHub
     Security Advisory. Note that CI will run after the advisory's changes are
     merged in on `main`.
   * For the backported release branches you should either create a PR targeting
     these branches or push the changes to the previous version-bump PRs.
3. **Merge the version-bump PR**
   * Like the patch release process this will kick everything else into motion.
     Note that the actual security fixes should be merged either before or as
     part of this PR.

After a security release has been made you'll also want to double-check that
the release notes on the branch are in sync with the `main` branch.
