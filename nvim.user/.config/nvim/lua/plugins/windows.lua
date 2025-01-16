if vim.loop.os_uname().sysname == "Windows_NT" then
  return {
    "equalsraf/neovim-gui-shim",
  }
else
  return {}
end
