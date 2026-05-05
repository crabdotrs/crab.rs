local M = {}

function M.setup(opts)
  opts = opts or {}

  require("crab.lsp").setup(opts.lsp or {})
  require("crab.treesitter").setup(opts.treesitter or {})
end

return M
