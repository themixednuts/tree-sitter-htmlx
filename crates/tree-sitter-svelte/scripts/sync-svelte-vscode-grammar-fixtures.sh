#!/usr/bin/env bash
set -euo pipefail

upstream_repository="https://github.com/sveltejs/language-tools.git"
upstream_ref="${1:-master}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
crate_dir="$(cd -- "${script_dir}/.." && pwd)"
fixtures_dir="${crate_dir}/tests/fixtures/svelte-vscode-grammar"
samples_dir="${fixtures_dir}/samples"

temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT

git -C "${temporary_dir}" init --quiet
git -C "${temporary_dir}" remote add origin "${upstream_repository}"
git -C "${temporary_dir}" fetch --depth 1 origin "${upstream_ref}"
git -C "${temporary_dir}" checkout --quiet FETCH_HEAD

rm -rf "${samples_dir}"
mkdir -p "${fixtures_dir}"
cp -R \
  "${temporary_dir}/packages/svelte-vscode/test/grammar/samples" \
  "${samples_dir}"

resolved_commit="$(git -C "${temporary_dir}" rev-parse HEAD)"
cat > "${fixtures_dir}/UPSTREAM.md" <<EOF
# Svelte VS Code Grammar Fixtures

These fixtures are copied from the official Svelte language-tools repository:

- Repository: \`https://github.com/sveltejs/language-tools\`
- Path: \`packages/svelte-vscode/test/grammar/samples\`
- Commit: \`${resolved_commit}\`

Keep the copied \`samples\` directory unchanged so it can be refreshed from
upstream and used as a stable parser regression corpus.

To refresh the fixtures:

\`\`\`sh
crates/tree-sitter-svelte/scripts/sync-svelte-vscode-grammar-fixtures.sh
\`\`\`
EOF
