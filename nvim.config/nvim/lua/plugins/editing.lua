---@type LazySpec
return {
  { "max397574/better-escape.nvim", enabled = false },
  {
    "rainbowhxch/accelerated-jk.nvim",
    config = function()
      vim.api.nvim_set_keymap("n", "j", "<Plug>(accelerated_jk_gj)", {})
      vim.api.nvim_set_keymap("n", "k", "<Plug>(accelerated_jk_gk)", {})
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
