#!/usr/bin/env bash
# End-to-end test script for sip-trunk.
# Starts sip-trunk-server and sip-trunk-api, exercises all management and webhook
# endpoints via curl, then verifies the CLI, and tears everything down.

set -euo pipefail

PASS=0
FAIL=0
ERRORS=()

# ── Ports (must match .env) ─────────────────────────────────────────────────
SERVER_PORT=3320
WEBHOOK_PORT=3301
MGMT_PORT=3302

# Use a known test secret for this run
export API_SECRET_KEY="e2e_test_secret_$(date +%s)"

SERVER_URL="http://127.0.0.1:${SERVER_PORT}"
WEBHOOK_URL="http://127.0.0.1:${WEBHOOK_PORT}"
MGMT_URL="http://127.0.0.1:${MGMT_PORT}"

SERVER_PID=""
API_PID=""

# ── Helpers ──────────────────────────────────────────────────────────────────

assert_eq() {
  local label="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    echo "  PASS  $label"
    PASS=$((PASS + 1))
  else
    echo "  FAIL  $label  (expected=$expected  got=$actual)"
    FAIL=$((FAIL + 1))
    ERRORS+=("$label")
  fi
}

assert_contains() {
  local label="$1" needle="$2" haystack="$3"
  if echo "$haystack" | grep -qF "$needle"; then
    echo "  PASS  $label"
    PASS=$((PASS + 1))
  else
    echo "  FAIL  $label  (expected to contain '$needle')"
    FAIL=$((FAIL + 1))
    ERRORS+=("$label")
  fi
}

wait_for_port() {
  local port="$1" name="$2" tries=0
  until curl -sf "http://127.0.0.1:${port}/calls" > /dev/null 2>&1 \
        || curl -o /dev/null -s -w "%{http_code}" "http://127.0.0.1:${port}/calls" 2>/dev/null | grep -q "^[24]"; do
    tries=$((tries + 1))
    [ "$tries" -gt 30 ] && { echo "Timed out waiting for $name on :$port"; cleanup; exit 1; }
    sleep 0.3
  done
  echo "  ready  $name (:$port)"
}

cleanup() {
  [ -n "$SERVER_PID" ] && kill "$SERVER_PID" 2>/dev/null || true
  [ -n "$API_PID" ]    && kill "$API_PID"    2>/dev/null || true
  wait 2>/dev/null || true
}
trap cleanup EXIT

# ── Build ────────────────────────────────────────────────────────────────────
echo
echo "=== Building workspace ==="
cargo build --workspace --quiet

# ── Start services ───────────────────────────────────────────────────────────
echo
echo "=== Starting services ==="

./target/debug/sip-trunk-server > /tmp/sip-server.log 2>&1 &
SERVER_PID=$!

./target/debug/sip-trunk-api > /tmp/sip-api.log 2>&1 &
API_PID=$!

wait_for_port "$SERVER_PORT"  "sip-trunk-server"
wait_for_port "$MGMT_PORT"    "sip-trunk-api (mgmt)"

# ── Management API tests ─────────────────────────────────────────────────────
echo
echo "=== Management API ==="

status=$(curl -s -o /dev/null -w "%{http_code}" "${MGMT_URL}/calls")
assert_eq "GET /calls — no auth → 401" "401" "$status"

status=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer wrong_secret" "${MGMT_URL}/calls")
assert_eq "GET /calls — wrong token → 401" "401" "$status"

resp=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer ${API_SECRET_KEY}" "${MGMT_URL}/calls")
status=$(echo "$resp" | tail -1)
body=$(echo "$resp" | head -1)
assert_eq "GET /calls — valid auth → 200" "200" "$status"
assert_contains "GET /calls — returns JSON array" "[]" "$body"

# ── Webhook endpoint tests ────────────────────────────────────────────────────
echo
echo "=== Webhook endpoint ==="

status=$(curl -s -o /dev/null -w "%{http_code}" -X POST "${WEBHOOK_URL}/webhooks/telnyx" \
  -H "content-type: application/json" \
  -H "telnyx-signature-ed25519: dGVzdA==" \
  -d '{}')
assert_eq "POST /webhooks/telnyx — missing timestamp → 400" "400" "$status"

status=$(curl -s -o /dev/null -w "%{http_code}" -X POST "${WEBHOOK_URL}/webhooks/telnyx" \
  -H "content-type: application/json" \
  -H "telnyx-timestamp: 1700000000" \
  -d '{}')
assert_eq "POST /webhooks/telnyx — missing signature → 400" "400" "$status"

status=$(curl -s -o /dev/null -w "%{http_code}" -X POST "${WEBHOOK_URL}/webhooks/telnyx" \
  -H "content-type: application/json" \
  -H "telnyx-timestamp: 1700000000" \
  -H "telnyx-signature-ed25519: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" \
  -d '{"data":{"event_type":"call.initiated"}}')
assert_eq "POST /webhooks/telnyx — invalid signature → 401" "401" "$status"

# ── CLI tests ─────────────────────────────────────────────────────────────────
echo
echo "=== CLI ==="

cli_out=$(./target/debug/sip-trunk-cli \
  --api-url "${MGMT_URL}" \
  --api-key "${API_SECRET_KEY}" \
  calls list 2>&1)
assert_contains "sip-trunk-cli calls list — succeeds" "[]" "$cli_out"

cli_exit=0
./target/debug/sip-trunk-cli \
  --api-url "${MGMT_URL}" \
  --api-key "wrong_secret" \
  calls list > /dev/null 2>&1 || cli_exit=$?
assert_eq "sip-trunk-cli calls list — wrong key → non-zero exit" "1" "$cli_exit"

# ── Server direct (management endpoints forwarded through) ────────────────────
echo
echo "=== Server reachable via management proxy ==="

# hangup unknown call — expect 404 proxied back
status=$(curl -s -o /dev/null -w "%{http_code}" \
  -X POST \
  -H "Authorization: Bearer ${API_SECRET_KEY}" \
  "${MGMT_URL}/calls/00000000-0000-0000-0000-000000000000/hangup")
assert_eq "POST /calls/<unknown-uuid>/hangup → 404" "404" "$status"

# ── Summary ──────────────────────────────────────────────────────────────────
echo
echo "=== Results ==="
echo "  Passed: $PASS"
echo "  Failed: $FAIL"
if [ ${#ERRORS[@]} -gt 0 ]; then
  echo
  echo "Failed tests:"
  for e in "${ERRORS[@]}"; do
    echo "  - $e"
  done
fi
echo

if [ "$FAIL" -gt 0 ]; then
  echo "Service logs:"
  echo "--- sip-trunk-server ---"
  cat /tmp/sip-server.log
  echo "--- sip-trunk-api ---"
  cat /tmp/sip-api.log
  exit 1
fi

echo "All e2e tests passed."
