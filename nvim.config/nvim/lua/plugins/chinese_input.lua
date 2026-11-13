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
    build = ":call jieba_vim#install()",
    init = function()
      vim.g.jieba_vim_lazy = 1
      vim.g.jieba_vim_keymap = 1
    end,
  },
}
