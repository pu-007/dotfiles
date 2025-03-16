-- https://github.com/LazyVim/LazyVim/blob/main/lua/lazyvim/config/options.lua

local opt = vim.opt

opt.scrolloff = 8 -- Lines of context
opt.wrap = true
opt.colorcolumn = { 80 }

-- vim.g.lazyvim_python_lsp = "pyright"
-- vim.g.lazyvim_python_ruff = "ruff"

-- views can only be fully collapsed with the global statusline
vim.opt.laststatus = 3

vim.g.python3_host_prog = "/usr/bin/python3"

vim.g.lazyvim_picker = "fzf"
