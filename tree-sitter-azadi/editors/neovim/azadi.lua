-- editors/neovim/azadi.lua
--
-- Registers the azadi tree-sitter parser and filetype.
-- Installed by editors/neovim/install.py to:
--   ~/.config/nvim/after/plugin/azadi.lua
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

-- Path to tree-sitter-azadi/ inside the azadi checkout.
-- install.py substitutes __GRAMMAR_DIR__ with the actual absolute path.
local grammar_dir = "__GRAMMAR_DIR__"

parsers.get_parser_configs().azadi = {
  install_info = {
    url                          = grammar_dir,
    files                        = { "src/parser.c" },
    generate_requires_npm        = false,
    requires_generate_from_grammar = false,
  },
  filetype = "azadi",
}

vim.filetype.add({
  extension = { azadi = "azadi" },
})
