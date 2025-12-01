# Release Process

This document describes how releases are automated for zeteo.

## Automated Release Pipeline

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Version Bump   │────▶│   PR Created    │────▶│   PR Merged     │────▶│  Auto Tag       │
│  (manual)       │     │   (automated)   │     │   (manual)      │     │  (automated)    │
└─────────────────┘     └─────────────────┘     └─────────────────┘     └─────────────────┘
                                                                                │
                                                                                ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Published to   │◀────│  Publish to     │◀────│  Build Binaries │◀────│  Release        │
│  crates.io      │     │  crates.io      │     │  (multi-arch)   │     │  Created        │
└─────────────────┘     └─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Quick Release (One-Step)

To release a new version:

1. Go to **Actions** → **Version Bump & Release**
2. Click **Run workflow**
3. Select bump type (`patch`, `minor`, or `major`)
4. Optionally add a prerelease identifier (e.g., `alpha.1`)
5. Choose to either:
   - **Create PR** (recommended): Creates a PR for review, then auto-tags on merge
   - **Skip PR**: Directly commits and tags (use with caution)

## Manual Release

If you prefer manual control:

### 1. Bump Version
```bash
# Edit Cargo.toml manually
vim Cargo.toml

# Update Cargo.lock
cargo update --workspace

# Commit changes
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"
git push origin main
```

### 2. Create Tag
```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

### 3. Release Happens Automatically
Once the tag is pushed, the following happens automatically:
- CI checks run
- GitHub Release is created with changelog
- Binaries are built for all platforms
- Package is published to crates.io

## Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Push/PR to main | Tests, linting, formatting |
| `version-bump.yml` | Manual | Bump version and create release PR |
| `auto-tag.yml` | Release PR merge | Automatically create release tag |
| `release.yml` | Tag push | Build binaries, create GitHub release |
| `publish.yml` | Release published | Publish to crates.io |

## Version Scheme

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backward compatible
- **PATCH** (0.0.1): Bug fixes, backward compatible

Prerelease versions:
- `X.Y.Z-alpha.N`: Alpha releases
- `X.Y.Z-beta.N`: Beta releases
- `X.Y.Z-rc.N`: Release candidates

## Secrets Required

Ensure these secrets are configured in your repository:

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | Publishing to crates.io |
| `GITHUB_TOKEN` | Auto-configured, used for releases |

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
