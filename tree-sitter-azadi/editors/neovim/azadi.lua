-- editors/neovim/azadi.lua
--
-- Drop this file in your Neovim config, e.g.:
--   ~/.config/nvim/after/plugin/azadi.lua
-- or require it from your init.lua.
--
-- Prerequisites: nvim-treesitter installed.
-- The grammar is built from the local checkout (no internet needed after clone).
--
-- Usage:
--   :TSInstall azadi      (first time, or after grammar changes)
--   :TSUpdate azadi       (re-build after grammar.js changes)

local ok, parsers = pcall(require, "nvim-treesitter.parsers")
if not ok then
  vim.notify("nvim-treesitter not found — skipping azadi parser registration", vim.log.levels.WARN)
  return
end

-- Adjust this path if your azadi checkout is elsewhere.
local azadi_root = vim.fn.expand("~/_prj/azadi")

parsers.get_parser_configs().azadi = {
  install_info = {
    url            = azadi_root,
    files          = { "tree-sitter-azadi/src/parser.c" },
    location       = "tree-sitter-azadi",
    branch         = "main",
    generate_requires_npm = false,
    requires_generate_from_grammar = false,
  },
  filetype = "azadi",
}

-- Associate file extensions with the azadi filetype.
-- .azadi  — dedicated azadi files
-- .md     — treat as azadi when you want macro highlighting in Markdown
--           (comment out if you prefer the markdown parser for .md files)
vim.filetype.add({
  extension = {
    azadi = "azadi",
  },
})

-- Treesitter config: make sure azadi is in the ensure_installed list
-- or call :TSInstall azadi manually.
-- The highlight / injection modules are auto-enabled for recognised filetypes.
