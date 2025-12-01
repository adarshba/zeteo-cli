#!/bin/sh
#
# Setup git hooks for zeteo development
#

HOOKS_DIR="$(git rev-parse --show-toplevel)/hooks"
GIT_HOOKS_DIR="$(git rev-parse --show-toplevel)/.git/hooks"

echo "Setting up git hooks..."

# Install pre-commit hook
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    cp "$HOOKS_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
    chmod +x "$GIT_HOOKS_DIR/pre-commit"
    echo "Installed pre-commit hook"
fi

echo ""
echo "Git hooks installed successfully!"
echo ""
echo "The pre-commit hook will run:"
echo "  - cargo fmt --check"
echo "  - cargo clippy"
echo "  - cargo check"
