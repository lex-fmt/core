-- Headless test to exercise the lex.binary helper without hitting the network.
-- The script is executed via `nvim --headless -u NONE -l this_file.lua` by the
-- Bats suite and prints TEST_PASSED/TEST_FAILED markers for CI consumption.

local script_path = debug.getinfo(1, 'S').source:sub(2)
local test_dir = vim.fn.fnamemodify(script_path, ':p:h')
local plugin_dir = vim.fn.fnamemodify(test_dir, ':h')
vim.opt.rtp:prepend(plugin_dir)

local uv = vim.loop
local binary = require('lex.binary')

local is_windows = uv.os_uname().sysname:lower():find('windows') ~= nil
local suffix = is_windows and '.exe' or ''
local temp_roots = {}

local function temp_plugin_root()
  local dir = vim.fn.tempname()
  vim.fn.mkdir(dir, 'p')
  table.insert(temp_roots, dir)
  return dir
end

local function cleanup()
  for _, dir in ipairs(temp_roots) do
    pcall(vim.fn.delete, dir, 'rf')
  end
end

local function assert_true(value, msg)
  if not value then
    error(msg)
  end
end

local function make_fake_binary(path)
  local fh = assert(io.open(path, 'w'))
  fh:write('#!/bin/sh\nexit 0\n')
  fh:close()
  if not is_windows then
    vim.fn.setfperm(path, 'rwxr-xr-x')
  end
end

local tests = {}

local function add_test(name, fn)
  table.insert(tests, { name = name, run = fn })
end

add_test('reuses existing binary', function()
  local root = temp_plugin_root()
  local version = 'v9.9.9'
  local bin_dir = root .. '/bin'
  vim.fn.mkdir(bin_dir, 'p')
  local existing = string.format('%s/lex-lsp-%s%s', bin_dir, version, suffix)
  make_fake_binary(existing)
  binary._set_test_overrides({ plugin_root = root })
  local resolved = binary.ensure_binary(version)
  assert_true(resolved == existing, 'should reuse existing binary for ' .. version)
end)

add_test('selects architecture specific assets', function()
  local select_asset = binary._select_asset_for_testing
  assert_true(select_asset('darwin', 'arm64') == 'lex-macos-arm64.tar.gz', 'mac arm64 asset mismatch')
  assert_true(select_asset('darwin', 'x86_64') == 'lex-macos-amd64.tar.gz', 'mac amd64 asset mismatch')
  assert_true(select_asset('linux', 'aarch64') == 'lex-linux-arm64.tar.gz', 'linux arm64 asset mismatch')
  assert_true(select_asset('linux', 'x86_64') == 'lex-linux-amd64.tar.gz', 'linux amd64 asset mismatch')
  assert_true(select_asset('windows', 'arm64') == 'lex-windows-arm64.zip', 'windows arm64 asset mismatch')
  assert_true(select_asset('windows', 'amd64') == 'lex-windows-amd64.zip', 'windows amd64 asset mismatch')
end)

add_test('downloads missing version', function()
  local root = temp_plugin_root()
  local version = 'v1.2.3'
  local downloads = {}
  binary._set_test_overrides({
    plugin_root = root,
    download_release = function(tag, dest)
      table.insert(downloads, { tag = tag, dest = dest })
      make_fake_binary(dest)
      return dest
    end,
  })
  local resolved = binary.ensure_binary(version)
  assert_true(#downloads == 1, 'should trigger a download when binary is missing')
  assert_true(downloads[1].tag == version, 'download should use requested version')
  assert_true(resolved == downloads[1].dest, 'ensure_binary should return downloaded path')
  assert_true(vim.fn.filereadable(resolved) == 1, 'downloaded binary must exist')
end)

add_test('falls back to latest release', function()
  local root = temp_plugin_root()
  local requested = 'v-does-not-exist'
  local fallback = 'v0.1.0'
  local download_tags = {}
  local latest_calls = 0
  binary._set_test_overrides({
    plugin_root = root,
    download_release = function(tag, dest)
      table.insert(download_tags, tag)
      if tag == requested then
        return nil, 'missing'
      end
      make_fake_binary(dest)
      return dest
    end,
    latest_tag = function()
      latest_calls = latest_calls + 1
      return fallback
    end,
  })
  local resolved = binary.ensure_binary(requested)
  assert_true(latest_calls == 1, 'latest_tag should be consulted when primary download fails')
  assert_true(#download_tags == 2, 'should attempt download twice (requested + fallback)')
  assert_true(download_tags[1] == requested and download_tags[2] == fallback, 'download order mismatch')
  assert_true(resolved ~= nil, 'fallback download should succeed')
end)

add_test('propagates failure notifications', function()
  local root = temp_plugin_root()
  local version = 'vbroken'
  local notifications = {}
  local orig_notify = vim.notify
  vim.notify = function(msg, level, opts)
    table.insert(notifications, { msg = msg, level = level, opts = opts })
  end
  binary._set_test_overrides({
    plugin_root = root,
    download_release = function()
      return nil, 'network error'
    end,
    latest_tag = function()
      return nil
    end,
  })
  local resolved = binary.ensure_binary(version)
  vim.notify = orig_notify
  assert_true(resolved == nil, 'ensure_binary should return nil on unrecoverable failure')
  assert_true(#notifications >= 1, 'user should be notified when downloads fail')
  local level = notifications[1].level
  assert_true(level == vim.log.levels.ERROR, 'notification should be marked as error')
end)

local function run_tests()
  for _, test in ipairs(tests) do
    local ok, err = xpcall(test.run, debug.traceback)
    binary._reset_test_overrides()
    if not ok then
      cleanup()
      print('TEST_FAILED: ' .. test.name .. ' -> ' .. err)
      os.exit(1)
    end
  end
  cleanup()
  print('TEST_PASSED: binary manager scenarios')
  os.exit(0)
end

run_tests()
