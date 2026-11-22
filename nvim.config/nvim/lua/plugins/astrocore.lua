---@type LazySpec
return {
  "AstroNvim/astrocore",
  opts = {
    mappings = {
      n = {
        H = { "^", desc = "行首非空白字符" },
        L = { "$", desc = "行尾" },
        -- \ 留给 <LocalLeader>，避免再与单键命令共享前缀。
        ["\\"] = false,
        ["<Leader>\\"] = { "<Cmd>split<CR>", desc = "横向分屏" },
        ["<Leader>r"] = { desc = "运行文件" },
      },
    },
  },
}
