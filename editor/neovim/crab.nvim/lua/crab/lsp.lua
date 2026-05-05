local M = {}

local function get_lsp_path()
  local path = vim.fn.exepath("crab-lsp")
  if path ~= "" then
    return path
  end

  local cargo_home = os.getenv("CARGO_HOME") or (os.getenv("HOME") .. "/.cargo")
  local default_path = cargo_home .. "/bin/crab-lsp"

  if vim.fn.filereadable(default_path) == 1 then
    return default_path
  end

  return nil
end

function M.setup(opts)
  opts = opts or {}

  local lsp_path = opts.cmd or get_lsp_path()
  if not lsp_path then
    vim.notify("crab-lsp not found in PATH", vim.log.levels.WARN)
    return
  end

  local lspconfig = require("lspconfig")
  local configs = require("lspconfig.configs")

  if not configs.crab then
    configs.crab = {
      default_config = {
        cmd = { lsp_path },
        filetypes = { "crab" },
        root_dir = function(fname)
          return lspconfig.util.root_pattern("crab.toml", ".git")(fname)
            or vim.fn.getcwd()
        end,
        settings = {},
      },
    }
  end

  lspconfig.crab.setup(opts)
end

return M
