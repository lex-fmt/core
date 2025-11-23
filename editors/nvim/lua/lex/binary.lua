-- Binary manager used by the Neovim plugin. Responsible for downloading the
-- correct lex-lsp release asset into ${PLUGIN_ROOT}/bin/ and returning the path
-- so the LSP client can spawn it. Binaries are versioned (lex-lsp-vX.Y.Z) to
-- keep upgrades atomic and the download uses GitHub release artifacts
-- (tar.gz+zip). The helper falls back to the latest release if the requested
-- version cannot be downloaded.

local uv = vim.loop
local M = {}

local OS_NAME = uv.os_uname().sysname:lower()
local IS_WINDOWS = OS_NAME:find('windows') ~= nil

local PLATFORM_ASSET = {
  linux = 'lex-linux-amd64.tar.gz',
  darwin = 'lex-macos-amd64.tar.gz',
  windows = 'lex-windows-amd64.zip',
}

local function select_asset()
  if OS_NAME:find('linux') then
    return PLATFORM_ASSET.linux
  elseif OS_NAME:find('darwin') then
    return PLATFORM_ASSET.darwin
  elseif IS_WINDOWS then
    return PLATFORM_ASSET.windows
  end
  return nil
end

local function run_cmd(cmd)
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
  local source = debug.getinfo(1, 'S').source:sub(2)
  return vim.fn.fnamemodify(source, ':h:h:h')
end

local function download_release(tag, dest)
  local asset = select_asset()
  if not asset then
    return nil, 'unsupported platform for automatic lex-lsp download'
  end

  local base = 'https://github.com/arthur-debert/lex/releases/download/%s/%s'
  local url = string.format(base, tag, asset)

  local tmpdir = with_tempdir()
  local archive = tmpdir .. '/' .. asset

  local _, curl_err = run_cmd({ 'curl', '-sSL', '-o', archive, url })
  if curl_err then
    return nil, curl_err
  end

  local extract_err
  if asset:match('%.tar%.gz$') then
    _, extract_err = run_cmd({ 'tar', '-xzf', archive, '-C', tmpdir })
  else
    local expand_cmd = string.format(
      'powershell -NoProfile -ExecutionPolicy Bypass -Command "Expand-Archive -Path \"%s\" -DestinationPath \"%s\" -Force"',
      archive,
      tmpdir
    )
    _, extract_err = run_cmd(expand_cmd)
  end
  if extract_err then
    return nil, extract_err
  end

  local binary_name = IS_WINDOWS and 'lex-lsp.exe' or 'lex-lsp'
  local extracted = tmpdir .. '/' .. binary_name
  if vim.fn.filereadable(extracted) == 0 then
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

return M
