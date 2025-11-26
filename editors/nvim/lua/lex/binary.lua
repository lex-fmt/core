-- Binary manager used by the Neovim plugin. Responsible for downloading the
-- correct lex-lsp release asset into ${PLUGIN_ROOT}/bin/ and returning the path
-- so the LSP client can spawn it. Binaries are versioned (lex-lsp-vX.Y.Z) to
-- keep upgrades atomic and the download uses GitHub release artifacts
-- (tar.gz+zip). The helper falls back to the latest release if the requested
-- version cannot be downloaded.

local uv = vim.loop
local M = {}
local test_overrides -- populated by _set_test_overrides during tests

local run_cmd

local UNAME = uv.os_uname()
local OS_NAME = UNAME.sysname:lower()
local MACHINE = (UNAME.machine or ''):lower()
local IS_WINDOWS = OS_NAME:find('windows') ~= nil

local PLATFORM_ASSETS = {
  linux = {
    amd64 = { filename = 'lex-lsp-x86_64-unknown-linux-gnu.tar.xz', kind = 'tar.xz' },
    arm64 = { filename = 'lex-lsp-aarch64-unknown-linux-gnu.tar.xz', kind = 'tar.xz' },
  },
  darwin = {
    amd64 = { filename = 'lex-lsp-x86_64-apple-darwin.tar.xz', kind = 'tar.xz' },
    arm64 = { filename = 'lex-lsp-aarch64-apple-darwin.tar.xz', kind = 'tar.xz' },
  },
  windows = {
    amd64 = { filename = 'lex-lsp-x86_64-pc-windows-msvc.zip', kind = 'zip' },
    arm64 = { filename = 'lex-lsp-aarch64-pc-windows-msvc.zip', kind = 'zip' },
  },
}

local function normalized_arch(machine)
  if not machine or machine == '' then
    return 'amd64'
  end
  machine = machine:lower()
  if machine:find('arm64', 1, true) or machine:find('aarch64', 1, true) then
    return 'arm64'
  end
  return 'amd64'
end

local function select_asset_for(os_name, machine)
  os_name = (os_name or ''):lower()
  local arch = normalized_arch(machine)
  if os_name:find('linux') then
    return PLATFORM_ASSETS.linux[arch] or PLATFORM_ASSETS.linux.amd64
  elseif os_name:find('darwin') then
    return PLATFORM_ASSETS.darwin[arch] or PLATFORM_ASSETS.darwin.amd64
  elseif os_name:find('windows') then
    return PLATFORM_ASSETS.windows[arch] or PLATFORM_ASSETS.windows.amd64
  end
  return nil
end

local function select_asset()
  return select_asset_for(OS_NAME, MACHINE)
end

local function extract_archive(archive, tmpdir, asset)
  local function tar_extract(args)
    return run_cmd(args)
  end

  if asset.kind == 'tar.xz' then
    local _, err = tar_extract({ 'tar', '-xJf', archive, '-C', tmpdir })
    if err then
      local fallback = string.format(
        "xz -dc %s | tar -xf - -C %s",
        vim.fn.shellescape(archive),
        vim.fn.shellescape(tmpdir)
      )
      _, err = run_cmd({ 'sh', '-c', fallback })
    end
    if err then
      return nil, err
    end
  elseif asset.kind == 'tar.gz' then
    local _, err = tar_extract({ 'tar', '-xzf', archive, '-C', tmpdir })
    if err then
      local fallback = string.format(
        "gzip -dc %s | tar -xf - -C %s",
        vim.fn.shellescape(archive),
        vim.fn.shellescape(tmpdir)
      )
      _, err = run_cmd({ 'sh', '-c', fallback })
    end
    if err then
      return nil, err
    end
  else
    local expand_cmd = string.format(
      'powershell -NoProfile -ExecutionPolicy Bypass -Command "Expand-Archive -Path \\\"%s\\\" -DestinationPath \\\"%s\\\" -Force"',
      archive,
      tmpdir
    )
    local _, err = run_cmd(expand_cmd)
    if err then
      return nil, err
    end
  end
  return true, nil
end

local function find_extracted_binary(tmpdir, binary_name)
  local paths = vim.fs.find(function(name)
    return name == binary_name
  end, { path = tmpdir, type = 'file', limit = 1 })
  return paths[1]
end

run_cmd = function(cmd)
  local output = vim.fn.system(cmd)
  if vim.v.shell_error ~= 0 then
    return nil, output
  end
  return output, nil
end

local function ensure_dir(path)
  if vim.fn.isdirectory(path) == 0 then
    vim.fn.mkdir(path, 'p')
  end
end

local function with_tempdir()
  local tmp = vim.fn.tempname()
  ensure_dir(tmp)
  return tmp
end

local function get_plugin_root()
  if test_overrides and test_overrides.plugin_root then
    return test_overrides.plugin_root
  end
  local source = debug.getinfo(1, 'S').source:sub(2)
  return vim.fn.fnamemodify(source, ':h:h:h')
end

local function download_release(tag, dest)
  if test_overrides and test_overrides.download_release then
    return test_overrides.download_release(tag, dest)
  end
  local asset = select_asset()
  if not asset then
    return nil, 'unsupported platform for automatic lex-lsp download'
  end

  local base = 'https://github.com/arthur-debert/lex/releases/download/%s/%s'
  local url = string.format(base, tag, asset.filename)

  local tmpdir = with_tempdir()
  local archive = tmpdir .. '/' .. asset.filename

  local token = vim.env.LEX_GITHUB_TOKEN or vim.env.GITHUB_TOKEN
  local curl_cmd = { 'curl', '-f', '-sSL', '-o', archive }
  if token and token ~= '' then
    table.insert(curl_cmd, '-H')
    table.insert(curl_cmd, 'Authorization: Bearer ' .. token)
  end
  table.insert(curl_cmd, url)

  local _, curl_err = run_cmd(curl_cmd)
  if curl_err then
    return nil, curl_err
  end

  local _, extract_err = extract_archive(archive, tmpdir, asset)
  if extract_err then
    return nil, extract_err
  end

  local binary_name = IS_WINDOWS and 'lex-lsp.exe' or 'lex-lsp'
  local extracted = find_extracted_binary(tmpdir, binary_name)
  if not extracted or vim.fn.filereadable(extracted) == 0 then
    vim.fn.delete(tmpdir, 'rf')
    return nil, 'lex-lsp binary not found in archive'
  end

  ensure_dir(vim.fn.fnamemodify(dest, ':h'))
  if vim.loop.fs_stat(dest) then
    pcall(vim.loop.fs_unlink, dest)
  end
  local ok, rename_err = os.rename(extracted, dest)
  vim.fn.delete(tmpdir, 'rf')
  if not ok then
    return nil, rename_err
  end

  if not IS_WINDOWS then
    vim.fn.setfperm(dest, 'rwxr-xr-x')
  end

  return dest, nil
end

local function latest_tag()
  if test_overrides and test_overrides.latest_tag then
    return test_overrides.latest_tag()
  end
  local api_url = 'https://api.github.com/repos/arthur-debert/lex/releases/latest'
  local output, err = run_cmd({ 'curl', '-sSL', api_url })
  if err then
    return nil
  end
  local ok, json = pcall(vim.json.decode, output)
  if not ok or not json.tag_name then
    return nil
  end
  return json.tag_name
end

local function ensure_binary(version)
  if not version or version == '' then
    return nil
  end

  -- If version is a path to an existing file, use it directly
  if vim.fn.filereadable(version) == 1 then
    return version
  end

  local plugin_root = get_plugin_root()
  local bin_dir = plugin_root .. '/bin'
  ensure_dir(bin_dir)

  local suffix = IS_WINDOWS and '.exe' or ''
  local filename = string.format('lex-lsp-%s%s', version, suffix)
  local binary_path = bin_dir .. '/' .. filename

  if vim.fn.filereadable(binary_path) == 1 then
    return binary_path
  end

  local tag = version
  local path, err = download_release(tag, binary_path)
  if not path then
    local fallback_tag = latest_tag()
    if fallback_tag and fallback_tag ~= tag then
      path, err = download_release(fallback_tag, binary_path)
      if path then
        vim.notify(
          string.format('lex-lsp %s unavailable, downloaded %s instead', version, fallback_tag),
          vim.log.levels.WARN,
          { title = 'Lex' }
        )
        return path
      end
    end
    vim.notify(
      string.format('Failed to download lex-lsp %s: %s', version, err or 'unknown error'),
      vim.log.levels.ERROR,
      { title = 'Lex' }
    )
    return nil
  end

  return path
end

M.ensure_binary = ensure_binary
M._set_test_overrides = function(overrides)
  test_overrides = overrides
end
M._reset_test_overrides = function()
  test_overrides = nil
end
M._select_asset_for_testing = select_asset_for
M._find_binary_in_tmpdir_for_testing = find_extracted_binary

return M
