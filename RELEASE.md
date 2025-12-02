# Release Process

This document describes how releases are automated for zeteo.

## Semantic Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backward compatible
- **PATCH** (0.0.1): Bug fixes, backward compatible

Prerelease versions:

- `X.Y.Z-alpha.N`: Alpha releases
- `X.Y.Z-beta.N`: Beta releases
- `X.Y.Z-rc.N`: Release candidates

## Conventional Commits

We use [Conventional Commits](https://www.conventionalcommits.org/) for automated changelog generation and version bumping:

| Commit Type        | Version Bump | Example                                    |
| ------------------ | ------------ | ------------------------------------------ |
| `feat:`            | MINOR        | `feat(cli): add log filtering`             |
| `fix:`             | PATCH        | `fix(api): handle timeout errors`          |
| `BREAKING CHANGE:` | MAJOR        | `feat!: remove deprecated endpoints`       |

See [CONTRIBUTING.md](CONTRIBUTING.md#commit-message-format) for the full commit message format guide.

## Automated Release Pipeline

The release process is streamlined into two workflows:

```
┌─────────────────┐                              ┌─────────────────┐
│   CI Workflow   │                              │ Release Workflow│
│   (automated)   │                              │  (tag-triggered)│
│                 │                              │                 │
│ • Build & Test  │                              │ • CI Checks     │
│ • Clippy        │                              │ • Build Bins    │
│ • Format Check  │                              │ • Create Release│
│ • Security Audit│                              │ • Publish Crate │
└─────────────────┘                              └─────────────────┘
      ▲                                                   ▲
      │                                                   │
  Push/PR to main                                   Git tag push
```

## Release Process

There are two ways to create a release:

### Option A: Manual Release (Recommended)

This gives you full control over the release process.

#### 1. Update Version

Edit `Cargo.toml` to bump the version:

```bash
vim Cargo.toml
```

Update the lock file:

```bash
cargo update --workspace
```

Commit the changes:

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"
git push origin main
```

#### 2. Create and Push Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

#### 3. Automated Steps

Once the tag is pushed, the release workflow automatically:

- Validates the version matches Cargo.toml
- Runs all CI checks (build, test, clippy, fmt)
- Builds cross-platform binaries (Linux, macOS, Windows)
- Creates a GitHub release with changelog
- Publishes to crates.io (stable releases only)

### Option B: Using GitHub Actions UI

You can also trigger a release directly from GitHub:

1. Go to **Actions** → **Release**
2. Click **Run workflow**
3. Enter the version (e.g., `1.0.0`)
4. Select release type (`release` or `prerelease`)
5. Click **Run workflow**

This will:
- Create the git tag automatically
- Run the same automated steps as Option A

## GitHub Actions Workflows

| Workflow     | Trigger         | Purpose                                                                   |
| ------------ | --------------- | ------------------------------------------------------------------------- |
| `ci.yml`     | Push/PR to main | Automated testing, linting, formatting checks, and security audit         |
| `release.yml`| Tag push/Manual | CI validation, cross-platform builds, GitHub release, crates.io publishing|

## Secrets Required

Ensure these secrets are configured in your repository:

| Secret                 | Purpose                            |
| ---------------------- | ---------------------------------- |
| `CARGO_REGISTRY_TOKEN` | Publishing to crates.io            |
| `GITHUB_TOKEN`         | Auto-configured, used for releases |

## Troubleshooting

### Release workflow failed

1. Check the Actions tab for error details
2. Common issues:
   - Version mismatch between Cargo.toml and tag
   - Missing `CARGO_REGISTRY_TOKEN` secret
   - Build failures on specific platforms

### Publish to crates.io failed

1. Ensure the version doesn't already exist on crates.io
2. Verify `CARGO_REGISTRY_TOKEN` is valid
3. Check `cargo package` works locally

### Tag already exists

Delete the tag and recreate:

```bash
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```
