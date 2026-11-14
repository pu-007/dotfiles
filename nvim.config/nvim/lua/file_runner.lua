-- 单文件运行辅助：为 code_runner.nvim 构建各语言的运行命令，
-- 并用 snacks 终端复用同一个窗口（<Leader>rr 运行、<Leader>rt 切换显示）。
local M = { term = nil }

local function pick_executable(names)
  for _, name in ipairs(names) do
    local path = vim.fn.exepath(name)
    if path ~= "" then return path end
  end
end

-- 优先使用 venv-selector 选中的解释器，其次 PATH 中的 python3/python
local function pick_python()
  local ok, venv_selector = pcall(require, "venv-selector")
  if ok and venv_selector.python then
    local python = venv_selector.python()
    if python and python ~= "" then return python end
  end
  return pick_executable { "python3", "python" }
end

-- 返回 shell 片段：找到解释器则为其加引号的路径，否则输出提示（$end 避免插件再追加文件路径）
local function interpreter(path, label)
  if path then return vim.fn.shellescape(path) end
  return "echo " .. vim.fn.shellescape("未找到可用的 " .. label .. "，请先安装或将其加入 PATH") .. " $end"
end

-- 编译到唯一临时产物，编译成功才运行（&& 保证失败时不执行旧产物）
local function compile_and_run(compiler)
  local output = vim.fn.tempname()
  return "cd $dir && " .. compiler .. " $file -o " .. vim.fn.shellescape(output) .. " && " .. vim.fn.shellescape(output)
end

--- code_runner.nvim 的 filetype 命令表；$file/$dir 由插件负责转义展开
function M.commands()
  return {
    python = function() return "cd $dir && " .. interpreter(pick_python(), "Python 解释器") .. " -u $file" end,
    c = function() return compile_and_run "gcc" end,
    cpp = function() return compile_and_run "g++" end,
    rust = function() return compile_and_run "rustc" end,
    javascript = "cd $dir && node $file",
    sh = "cd $dir && bash $file",
    bash = "cd $dir && bash $file",
    lua = function()
      return "cd $dir && " .. interpreter(pick_executable { "lua", "luajit" }, "Lua 解释器") .. " $file"
    end,
    -- TypeScript 运行时二选一（取消注释后生效）：
    -- typescript = "cd $dir && npx tsx $file", -- 需要项目内安装 tsx
    -- typescriptreact = "cd $dir && npx tsx $file",
    -- typescript = "cd $dir && deno run $file", -- Deno 项目
  }
end

local function stop_previous()
  local term = M.term
  M.term = nil
  if not (term and term.buf_valid and term:buf_valid()) then return end
  local buf = term.buf
  local chan = vim.b[buf].terminal_job_id
  if chan then pcall(vim.fn.jobstop, chan) end
  pcall(function() term:close() end)
  if vim.api.nvim_buf_is_valid(buf) then pcall(vim.api.nvim_buf_delete, buf, { force = true }) end
end

--- 运行当前文件：先保存，再在（复用的）snacks 浮动终端中执行
function M.run()
  if vim.bo.buftype ~= "" or vim.fn.expand "%:p" == "" then
    return vim.notify("请先把文件保存到磁盘再运行", vim.log.levels.WARN)
  end
  if vim.bo.modified then
    local ok = pcall(vim.cmd.update)
    if not ok then return vim.notify("保存失败，已取消运行", vim.log.levels.ERROR) end
  end
  local command = require("code_runner").get_filetype_command()
  if not command or command == "" then
    return vim.notify(("还没有为 %s 配置运行命令（见 lua/file_runner.lua）"):format(vim.bo.filetype), vim.log.levels.WARN)
  end
  stop_previous()
  M.term = require("snacks.terminal").open(command, { auto_close = false, start_insert = false, auto_insert = false })
end

--- 显示/隐藏最近一次的运行终端（程序退出后输出仍保留）
function M.toggle()
  local term = M.term
  if term and term.buf_valid and term:buf_valid() then
    term:toggle()
  else
    vim.notify("还没有运行过任何文件", vim.log.levels.WARN)
  end
end

return M
