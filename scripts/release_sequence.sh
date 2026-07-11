#!/usr/bin/env bash
#
# Automated release helper for MangoFetch crates
#
# Usage: ./scripts/release_sequence.sh <new_version>
#
# 1) Bumps the version in every Cargo.toml (workspace root).
# 2) Runs cargo check and cargo test for the whole workspace.
# 3) Publishes the three crates in the correct order:
#    mangofetch-plugin-sdk → mangofetch-core → mangofetch
#
# Environment:
#   CARGO_REGISTRY_TOKEN – personal access token with publish rights
#
# Exit on any error unless CARGO_REGISTRY_TOKEN is unset (publish step is
# skipped then, allowing local verification only).

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <new_version>"
  exit 1
fi

NEW_VERSION="$1"

# -------------------------------------------------------------------------
# 1️⃣ Bump version in all Cargo.toml files
# -------------------------------------------------------------------------
echo "🔖 Bumping version to $NEW_VERSION"
find . -name Cargo.toml -exec sed -i '' -E "s/^(version = \".*\")/version = \"$NEW_VERSION\"/" {} +

# -------------------------------------------------------------------------
# 2️⃣ Verify the workspace compiles and passes tests
# -------------------------------------------------------------------------
echo "📦 Checking the workspace"
cargo check --workspace

echo "✅ Running tests"
cargo test --workspace --quiet

# -------------------------------------------------------------------------
# 3️⃣ Publish crates (requires CARGO_REGISTRY_TOKEN)
# -------------------------------------------------------------------------
if [[ -z "${CARGO_REGISTRY_TOKEN:-}" ]]; then
  echo "⚠️  CARGO_REGISTRY_TOKEN not set – publishing is skipped."
  echo "💡 Run with CARGO_REGISTRY_TOKEN set to actually publish."
  exit 0
fi

CRATES=("mangofetch-plugin-sdk" "mangofetch-core" "mangofetch")
for CRATE in "${CRATES[@]}"; do
  echo "🚀 Publishing $CRATE"
  cargo publish -p "$CRATE" --token "$CARGO_REGISTRY_TOKEN"
done

echo "🎉 All crates have been published successfully."