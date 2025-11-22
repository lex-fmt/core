#!/usr/bin/env bats

setup() {
    # Get the directory of the test file
    export SCRIPT_DIR="$BATS_TEST_DIRNAME"
    export PLUGIN_DIR="$(dirname "$SCRIPT_DIR")"
    export CONFIG_INIT="$PLUGIN_DIR/config/init.lua"
    export MINIMAL_INIT="$SCRIPT_DIR/minimal_init.lua"
    export NVIM_APPNAME="lex-test"
}

@test "Plugin loads successfully" {
    run nvim --headless -u "$CONFIG_INIT" -l "$SCRIPT_DIR/test_plugin_loads.lua"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "TEST_PASSED" ]]
}

@test "Filetype detection for .lex files" {
    run nvim --headless -u "$CONFIG_INIT" -l "$SCRIPT_DIR/test_filetype.lua"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "TEST_PASSED" ]]
}

@test "Theme highlights" {
    run nvim --headless -u "$CONFIG_INIT" -l "$SCRIPT_DIR/test_theme.lua"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "TEST_PASSED" ]]
}

@test "LSP hover functionality" {
    run nvim --headless -u "$MINIMAL_INIT" -l "$SCRIPT_DIR/test_lsp_hover.lua"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "TEST_PASSED" ]]
}

@test "LSP semantic tokens functionality" {
    run nvim --headless -u "$MINIMAL_INIT" -l "$SCRIPT_DIR/test_lsp_semantic_tokens.lua"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "TEST_PASSED" ]]
}

@test "LSP document symbols functionality" {
    run nvim --headless -u "$MINIMAL_INIT" -l "$SCRIPT_DIR/test_lsp_document_symbols.lua"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "TEST_PASSED" ]]
}

@test "LSP folding ranges functionality" {
    run nvim --headless -u "$MINIMAL_INIT" -l "$SCRIPT_DIR/test_lsp_folding_ranges.lua"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "TEST_PASSED" ]]
}

@test "Semantic token rendering enabled" {
    run nvim --headless -u "$MINIMAL_INIT" -l "$SCRIPT_DIR/test_semantic_rendering.lua"
    [ "$status" -eq 0 ]
    [[ "$output" =~ "TEST_PASSED" ]]
}

