# crab.nvim

Neovim plugin for the Crab programming language.

## Features

- Syntax highlighting (vim regex and tree-sitter)
- LSP support via crab-lsp
- Filetype detection for .crab files
- Proper indentation and formatting

## Installation

### Using lazy.nvim

```lua
{
  'crabdotrs/crab.nvim',
  dependencies = {
    'neovim/nvim-lspconfig',
    'nvim-treesitter/nvim-treesitter',
  },
  config = function()
    require('crab').setup({
      lsp = {
        cmd = nil, -- will auto-detect crab-lsp in PATH
      },
      treesitter = {
        parser_url = 'https://github.com/crabdotrs/tree-sitter-crab',
      }
    })
  end
}
```

### Using packer.nvim

```lua
use {
  'crabdotrs/crab.nvim',
  requires = {
    'neovim/nvim-lspconfig',
    'nvim-treesitter/nvim-treesitter',
  },
  config = function()
    require('crab').setup()
  end
}
```

## Requirements

- Neovim >= 0.7.0
- crab-lsp (for LSP features)
- nvim-treesitter (for tree-sitter highlighting)

## Configuration

```lua
require('crab').setup({
  lsp = {
    cmd = '/path/to/crab-lsp',
    on_attach = function(client, bufnr)
      -- your on_attach function
    end,
    capabilities = vim.lsp.protocol.make_client_capabilities(),
  },
  treesitter = {
    enable = true,
    parser_url = nil, -- use default
  }
})
```
