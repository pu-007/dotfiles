---@type LazySpec
return {
  { "max397574/better-escape.nvim", enabled = false },
  { "rainbowhxch/accelerated-jk.nvim" },
  {
    "AstroNvim/astrocore",
    opts = function(_, opts)
      -- AstroNvim 默认让 x.j/x.k 与 n.j/n.k 共用同一张表：静态 opts 会被深合并进两个模式，
      -- 而 accelerated-jk 的 <Plug> 只存在于普通模式，可视模式的 j/k 会因此失效。
      -- 这里整表替换普通模式映射，可视模式保留 AstroNvim 默认的 gj/gk。
      opts.mappings.n.j = { "<Plug>(accelerated_jk_gj)", desc = "向下移动（加速）" }
      opts.mappings.n.k = { "<Plug>(accelerated_jk_gk)", desc = "向上移动（加速）" }
    end,
  },
  {
    "L3MON4D3/LuaSnip",
    config = function(plugin, opts)
      require("luasnip").filetype_extend("javascript", { "javascriptreact" })
      require("astronvim.plugins.configs.luasnip")(plugin, opts)
    end,
  },
  {
    "windwp/nvim-autopairs",
    config = function(plugin, opts)
      require("astronvim.plugins.configs.nvim-autopairs")(plugin, opts)
      local Rule = require "nvim-autopairs.rule"
      local cond = require "nvim-autopairs.conds"
      require("nvim-autopairs").add_rules {
        -- LaTeX 数学分隔符：成对输入，转义的 \$ 不配对，$ 内按 <cr> 不自动换行
        Rule("$", "$", { "tex", "latex" })
          :with_pair(cond.not_before_regex "\\")
          :with_cr(cond.none()),
      }
    end,
  },
}
