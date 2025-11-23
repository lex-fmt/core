#!/usr/bin/env bats

setup() {
  export EXTENSION_DIR="$(cd "${BATS_TEST_DIRNAME}/.." && pwd)"
}

@test "VS Code extension npm test" {
  cd "$EXTENSION_DIR"
  run npm test
  if [ "$status" -ne 0 ]; then
    echo "$output" >&2
  fi
  [ "$status" -eq 0 ]
  [[ "$output" =~ "VSCode Integration" ]]
}
