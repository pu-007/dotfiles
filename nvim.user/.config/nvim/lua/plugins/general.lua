return {
  -- Lazy
  {
    "LazyVim/LazyVim",
    opts = {
      colorscheme = "gruvbox-material",
    },
  },
  { "sainnhe/gruvbox-material" },
  -- { "noearc/jieba.nvim", dependencies = { "noearc/jieba-lua" }, opts = {} },
  {
    "kkew3/jieba.vim",
    -- tag = "v1.0.4",
    build = "./build.sh",
    init = function()
      vim.g.jieba_vim_lazy = 1
      vim.g.jieba_vim_keymap = 1
    end,
  },
  -- TODO: An AI plugs to be configured
  -- https://github.com/olimorris/codecompanion.nvim
  -- https://github.com/yetone/avante.nvim
  {
    "nvim-treesitter/nvim-treesitter",
    opts = function(_, opts)
      -- add tsx and treesitter
      vim.list_extend(opts.ensure_installed, {
        "tsx",
        "typescript",
        "latex",
        -- python-pylatexenc for latex2text
      })
    end,
  },
}
