#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "$0")/.." && pwd)
report_dir=$(mktemp -d)
server_log=$(mktemp)
python3 -m http.server 8765 --directory "$repo_dir" >"$server_log" 2>&1 &
server_pid=$!
cleanup() {
  kill "$server_pid" 2>/dev/null || true
  rm -rf "$report_dir"
  rm -f "$server_log"
}
trap cleanup EXIT

for _ in $(seq 1 30); do
  if curl --fail --silent http://127.0.0.1:8765/fixtures/seeded.html >/dev/null; then break; fi
  sleep .1
done

set +e
cargo run --quiet -p zoomcheck -- check "$repo_dir/fixtures/seeded-workflow.json" --out "$report_dir" --quiet
exit_code=$?
set -e
if [[ $exit_code -ne 1 ]]; then
  echo "expected finding exit code 1, got $exit_code" >&2
  exit 1
fi

python3 - "$report_dir/report.json" <<'PY'
import json, sys
report = json.load(open(sys.argv[1], encoding="utf-8"))
assert [run["zoom"] for run in report["runs"]] == [200, 400]
for run in report["runs"]:
    width = float(run["viewportCss"].split("×")[0])
    expected = 1280 / (run["zoom"] / 100)
    assert abs(width - expected) < 40, (run["zoom"], width, expected)
    detected = false_positives = 0
    for index, step in enumerate(run["steps"]):
        failed = any(item["severity"] == "failure" for item in step["findings"])
        expected = f"#good-{index + 1}" if index < 10 else f"#bad-{index - 9}"
        assert step["selector"] == expected, (run["zoom"], index + 1, step["selector"], expected)
        if index >= 10 and failed: detected += 1
        if index < 10 and failed: false_positives += 1
    assert detected >= 9, (run["zoom"], detected)
    assert false_positives == 0, (run["zoom"], false_positives)
print("seeded zoom suite: >=90% detection, 0% false-positive failures")
PY
