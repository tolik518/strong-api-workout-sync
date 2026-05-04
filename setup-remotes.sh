#!/usr/bin/env bash
set -euo pipefail

GITHUB_URL="git@github.com:tolik518/strong-api-workout-sync.git"
TANGLED_URL="git@tangled.org:tolik518.tngl.sh/strong-api-workout-sync"

# Ensure we are inside a git repo
git rev-parse --is-inside-work-tree >/dev/null

# Ensure origin exists
if ! git remote get-url origin >/dev/null 2>&1; then
  git remote add origin "$GITHUB_URL"
else
  # Use GitHub as the default fetch/pull source
  git remote set-url origin "$GITHUB_URL"
fi

# Remove old/duplicate push URLs if present
git remote set-url --delete --push origin "$GITHUB_URL" 2>/dev/null || true
git remote set-url --delete --push origin "$TANGLED_URL" 2>/dev/null || true

# Add both push targets
git remote set-url --add --push origin "$GITHUB_URL"
git remote set-url --add --push origin "$TANGLED_URL"

echo "Configured remotes:"
git remote -v