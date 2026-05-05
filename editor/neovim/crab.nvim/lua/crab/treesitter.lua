local M = {}

function M.setup(opts)
  opts = opts or {}

  local parser_config = require("nvim-treesitter.parsers").get_parser_configs()

  parser_config.crab = {
    install_info = {
      url = opts.parser_url or "https://github.com/crabdotrs/tree-sitter-crab",
      files = { "src/parser.c" },
      branch = opts.branch or "main",
      generate_requires_npm = false,
      requires_generate_from_grammar = false,
    },
    filetype = "crab",
  }

  vim.treesitter.language.register("crab", "crab")
end

return M
