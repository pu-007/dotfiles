if vim.loop.os_uname().sysname == "Linux" then
  return {
    { "lambdalisue/suda.vim", cmd = { "SudaRead", "SudaWrite" } },
    -- {
    --   "xiyaowong/transparent.nvim",
    --   opts = {},
    -- },
  }
else
  return {}
end
