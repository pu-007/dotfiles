-- LSP 诊断忽略（规则见 lua/diagnostic_ignores.lua，:LspIgnoreEdit / :LspIgnoreReload）
require("diagnostic_filter").setup()

vim.keymap.set("n", "H", "^", { desc = "Home" })
vim.keymap.set("n", "L", "$", { desc = "End" })
