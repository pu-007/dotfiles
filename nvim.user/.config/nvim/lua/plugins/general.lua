return {
  -- Lazy
  {
    "LazyVim/LazyVim",
    opts = {
      colorscheme = "gruvbox-material",
    },
  },
  { "sainnhe/gruvbox-material" },
  {
    "kkew3/jieba.vim",
    -- tag = "v1.0.4",
    build = "./build.sh",
    init = function()
      vim.g.jieba_vim_lazy = 1
      vim.g.jieba_vim_keymap = 1
    end,
  },
  {
    "saghen/blink.cmp",
    dependencies = {
      { "Kaiser-Yang/blink-cmp-avante" },
      { "saghen/blink.compat" },
    },
    opts = function(_, opts)
      table.insert(opts.sources.default, "avante")
      opts.sources.providers.avante = {
        module = "blink-cmp-avante",
        name = "Avante",
        opts = {},
      }
    end,
  },
  {
    "yetone/avante.nvim",
    event = "VeryLazy",
    version = false, -- Never set this value to "*"! Never!
    keys = function(_, keys)
      ---@type avante.Config
      local opts =
        require("lazy.core.plugin").values(require("lazy.core.config").spec.plugins["avante.nvim"], "opts", false)

      local mappings = {
        {
          opts.mappings.ask,
          function()
            require("avante.api").ask()
          end,
          desc = "avante: ask",
          mode = { "n", "v" },
        },
        {
          opts.mappings.refresh,
          function()
            require("avante.api").refresh()
          end,
          desc = "avante: refresh",
          mode = "v",
        },
        {
          opts.mappings.edit,
          function()
            require("avante.api").edit()
          end,
          desc = "avante: edit",
          mode = { "n", "v" },
        },
        {
          "<leader>ip",
          function()
            return vim.bo.filetype == "AvanteInput" and require("avante.clipboard").paste_image()
              or require("img-clip").paste_image()
          end,
          desc = "clip: paste image",
        },
      }
      mappings = vim.tbl_filter(function(m)
        return m[1] and #m[1] > 0
      end, mappings)
      return vim.list_extend(mappings, keys)
    end,
    opts = {
      provider = "xiaohu",
      vendors = {
        xiaohu = {
          __inherited_from = "openai",
          endpoint = "https://xiaohumini.site/v1",
          model = "gemini-2.5-pro-exp-03-25",
        },
      },
      web_search_engine = {
        provider = "tavily",
      },
      mappings = {
        ask = "<leader>ia", -- ask
        edit = "<leader>ie", -- edit
        refresh = "<leader>ir", -- refresh
      },
    },
    -- if you want to build from source then do `make BUILD_FROM_SOURCE=true`
    build = "make",
    -- build = "powershell -ExecutionPolicy Bypass -File Build.ps1 -BuildFromSource false" -- for windows
    dependencies = {
      "nvim-treesitter/nvim-treesitter",
      "stevearc/dressing.nvim",
      "nvim-lua/plenary.nvim",
      "MunifTanjim/nui.nvim",
      {
        -- support for image pasting
        "HakonHarnes/img-clip.nvim",
        event = "VeryLazy",
        opts = {
          -- recommended settings
          default = {
            embed_image_as_base64 = false,
            prompt_for_file_name = false,
            drag_and_drop = {
              insert_mode = true,
            },
            -- required for Windows users
            use_absolute_path = true,
          },
        },
      },
      {
        -- Make sure to set this up properly if you have lazy=true
        "MeanderingProgrammer/render-markdown.nvim",
        opts = {
          file_types = { "markdown", "Avante" },
        },
        ft = { "markdown", "Avante" },
      },
    },
  },
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
  { "CRAG666/code_runner.nvim", config = true },
  {
    "CRAG666/betterTerm.nvim",
    opts = {
      position = "bot",
      size = 15,
    },
  },
}
