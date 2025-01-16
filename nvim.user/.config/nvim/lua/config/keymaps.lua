-- Keymaps are automatically loaded on the VeryLazy event
-- Default keymaps that are always set: https://github.com/LazyVim/LazyVim/blob/main/lua/lazyvim/config/keymaps.lua
-- Add any additional keymaps here

local map = LazyVim.safe_keymap_set

map("i", "<C-v>", "<C-r><C-p>+", { desc = "Paste in Insert Mode" })

-- I like using H and L for beginning/end of line
vim.keymap.set("n", "H", "^", { desc = "First non-whitespace character" })
vim.keymap.set("n", "L", "$", { desc = "Last non-whitespace character" })
