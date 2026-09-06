#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "$0")/.." && pwd)
consumer_dir=$(mktemp -d)
cleanup() {
  rm -rf "$consumer_dir"
}
trap cleanup EXIT

cd "$repo_dir"
cargo package -p zoomcheck --allow-dirty
crate_file=$(find target/package -maxdepth 1 -name 'zoomcheck-*.crate' -print -quit)
test -n "$crate_file"

tar -xzf "$crate_file" -C "$consumer_dir"
package_dir=$(find "$consumer_dir" -mindepth 1 -maxdepth 1 -type d -name 'zoomcheck-*' -print -quit)
install_root="$consumer_dir/install"
cargo install --path "$package_dir" --root "$install_root" --quiet

set +e
"$install_root/bin/zoomcheck" demo --out "$consumer_dir/report" --quiet
status=$?
set -e
test "$status" -eq 1
test -f "$consumer_dir/report/index.html"
test -f "$consumer_dir/report/report.json"
test -f "$consumer_dir/report/zoom-400.png"
python3 - "$consumer_dir/report/report.json" <<'PY'
import json, sys
report = json.load(open(sys.argv[1], encoding='utf-8'))
assert report['workflow'] == 'Checkout flyout sample'
assert [run['zoom'] for run in report['runs']] == [200, 400]
assert report['failures'] > 0
PY
echo 'clean consumer package: bundled demo report passed'
