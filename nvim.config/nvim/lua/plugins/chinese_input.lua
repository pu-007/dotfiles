---@type LazySpec
return {
  {
    "pu-007/im-select-ahk.nvim",
    event = { "InsertEnter", "InsertLeave", "CmdlineEnter", "CmdlineLeave", "ModeChanged" },
    opts = {
      toggle_key = "RShift", -- 回退模式的切换键
      ime_timeout = 500, -- SendMessage 超时时间 (ms)
      async = true,
      mode_config = {
        normal = "always_en", -- Normal 模式始终英文
        insert = "restore", -- Insert 模式记忆恢复
        cmdline = "restore", -- Command-line 模式记忆恢复
        search = "restore", -- 搜索模式记忆恢复
        visual = false, -- Visual 模式不干预
        replace = "restore", -- Replace 模式记忆恢复
        terminal = false, -- Terminal 模式不干预
        select = false, -- Select 模式不干预
      },
    },
  },
  {
    "kkew3/jieba.vim",
    tag = "v2.1.0",
    dependencies = { "tpope/vim-repeat" }, -- jieba 的 dw/ciw 等操作需要它支持 . 重复
    build = ":call jieba_vim#install()",
    init = function()
      vim.g.jieba_vim_lazy = 1
      vim.g.jieba_vim_keymap = 0 -- 统一由下方 AstroCore 注册，避免覆盖 g@ 兼容映射
    end,
  },
  {
    "AstroNvim/astrocore",
    opts = function(_, opts)
      local motions = { "w", "W", "e", "E", "b", "B", "ge", "gE", "iw", "iW", "aw", "aW" }
      for _, mode in ipairs { "n", "x", "o" } do
        opts.mappings[mode] = opts.mappings[mode] or {}
        for _, key in ipairs(motions) do
          if mode ~= "n" or not key:match "^[ia]" then
            local plug = "<Plug>(Jieba_" .. key .. ")"
            opts.mappings[mode][key] = {
              mode == "o" and function()
                -- jieba 会自行执行操作符并附加 Esc；g@ 必须由原生 motion 完成，
                -- 否则 surround 的后续包围符输入和点重复会被中断。
                return vim.v.operator == "g@" and key or plug
              end or plug,
              expr = mode == "o",
              desc = "中文分词 " .. key,
            }
          end
        end
      end
    end,
  },
}
