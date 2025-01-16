if vim.loop.os_uname().sysname == "Linux" then
  return {
    { "lambdalisue/suda.vim", cmd = { "SudaRead", "SudaWrite" } },
  }
else
  return {}
end
