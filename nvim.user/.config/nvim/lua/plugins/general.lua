return {
  -- Lazy
  {
    "LazyVim/LazyVim",
    opts = {
      colorscheme = "gruvbox-material",
    },
  },
  -- TODO update paredit when it supports treesitter main branch
  {
    "julienvincent/nvim-paredit",
    enabled = false,
  },
  -- TODO IME 自动选择 可以自动回到中文模式
  -- {
  --   "keaising/im-select.nvim",
  --   config = function()
  --     require("im_select").setup({
  --       default_im_select = "cn",
  --       default_command = "/mnt/c/Users/zion/Apps/im-select.exe",
  --     })
  --   end,
  -- },
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
  -- TODO: use lazy extra to config, better AI integration
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
      providers = {
        xiaohu = {
          __inherited_from = "openai",
          endpoint = "https://xiaohumini.site/v1",
          model = "gemini-2.5-flash-preview-05-20",
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
  { "CRAG666/code_runner.nvim", config = true },
  {
    "CRAG666/betterTerm.nvim",
    opts = {
      position = "bot",
      size = 15,
    },
  },
  {
    "chomosuke/typst-preview.nvim",
    -- lazy = false, -- or ft = 'typst'
    ft = "typst",
    -- version = "1.*",
    opts = {}, -- lazy.nvim will implicitly calls `setup {}`
  },
  {
    "stevearc/conform.nvim",
    opts = function()
      local plugin = require("lazy.core.config").plugins["conform.nvim"]
      local opts = {
        default_format_opts = {
          timeout_ms = 3000,
          async = false, -- not recommended to change
          quiet = false, -- not recommended to change
          lsp_format = "fallback", -- not recommended to change
        },
        formatters_by_ft = {
          lua = { "stylua" },
          fish = { "fish_indent" },
          sh = { "shfmt" },
          python = { "ruff", "yapf" },
        },
        -- The options you set here will be merged with the builtin formatters.
        -- You can also define any custom formatters here.
        ---@type table<string, conform.FormatterConfigOverride|fun(bufnr: integer): nil|conform.FormatterConfigOverride>
        formatters = {
          injected = { options = { ignore_errors = true } },
          -- # Example of using dprint only when a dprint.json file is present
          -- dprint = {
          --   condition = function(ctx)
          --     return vim.fs.find({ "dprint.json" }, { path = ctx.filename, upward = true })[1]
          --   end,
          -- },
          --
          -- # Example of using shfmt with extra args
          -- shfmt = {
          --   prepend_args = { "-i", "2", "-ci" },
          -- },
        },
      }
      return opts
    end,
  },
}
