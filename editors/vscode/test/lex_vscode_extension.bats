#!/usr/bin/env bats

setup() {
  export EXTENSION_DIR="$(cd "${BATS_TEST_DIRNAME}/.." && pwd)"
}

@test "VS Code extension npm test" {
  cd "$EXTENSION_DIR"
  run npm test
  echo "$output"
  [ "$status" -eq 0 ]
  [[ "$output" =~ "VSCode Integration" ]]
}
