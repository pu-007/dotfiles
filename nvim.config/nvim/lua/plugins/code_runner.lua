-- 单文件快速运行（code_runner.nvim）：命令表见 lua/file_runner.lua。
-- <Leader>rr 运行当前文件（先保存，复用同一个 snacks 终端）；
-- <Leader>rt 显示/隐藏运行终端；:RunFile 等命令为插件原生行为（每次新开终端）。
---@type LazySpec
return {
  {
    "CRAG666/code_runner.nvim",
    cmd = { "RunCode", "RunFile", "RunProject", "RunClose", "CRFiletype", "CRProjects" },
    keys = {
      { "<Leader>rr", function() require("file_runner").run() end, desc = "运行当前文件" },
      { "<Leader>rt", function() require("file_runner").toggle() end, desc = "切换运行终端" },
    },
    dependencies = { "folke/snacks.nvim" },
    opts = {
      mode = "snacks",
      before_run_filetype = function() vim.cmd.update() end,
      filetype = require("file_runner").commands(),
    },
  },
}
