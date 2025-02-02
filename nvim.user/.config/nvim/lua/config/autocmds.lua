vim.api.nvim_create_augroup("SaveRecentFiles", { clear = true })

vim.api.nvim_create_autocmd("VimLeave", {
  group = "SaveRecentFiles",
  callback = function()
    -- 获取最近文件列表
    local recent_files = vim.v.oldfiles
    local file_path = os.getenv("HOME") .. "/.config/nvim/recent_files.txt"

    -- 打开文件并写入最近文件列表
    local file, err = io.open(file_path, "w")
    if not file then
      print("Error opening file: " .. err)
      return
    end

    for _, file_name in ipairs(recent_files) do
      if file_name ~= "" then
        file:write(file_name .. "\n")
      end
    end

    -- 关闭文件
    file:close()
  end,
})
