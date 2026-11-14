-- 从配置根目录运行：nvim --headless -u NONE -i NONE -l tests/file_runner.lua
-- 加载真实配置并附加 UI，验证运行终端的按键、模式、进程和输出生命周期。
local root = vim.fn.tempname()
vim.fn.mkdir(root, "p")
local source = root .. "/runner.sh"
vim.fn.writefile({
  "printf 'runner_ready\\n'",
  "IFS= read -r value",
  "printf 'runner_input=%s\\n' \"$value\"",
}, source)

local channel = vim.fn.jobstart({ vim.v.progpath, "--embed", "--headless", "-i", "NONE" }, { rpc = true })
assert(channel > 0, "无法启动 Neovim 子进程")

local function remote(code, ...)
  return vim.fn.rpcrequest(channel, "nvim_exec_lua", code, { ... })
end

local function input(keys) vim.fn.rpcrequest(channel, "nvim_input", keys) end

local function state()
  return remote [[
    local runner = package.loaded.file_runner
    local term = runner and runner.term
    local valid = term and term:buf_valid()
    local job = valid and vim.b[term.buf].terminal_job_id
    return {
      buf = valid and term.buf or 0,
      visible = valid and term:valid() or false,
      current_buf = vim.api.nvim_get_current_buf(),
      mode = vim.fn.mode(),
      job = job or 0,
      exit = job and vim.fn.jobwait({ job }, 0)[1] or -3,
      lines = valid and vim.api.nvim_buf_get_lines(term.buf, 0, -1, false) or {},
    }
  ]]
end

local function wait_for(label, predicate)
  assert(vim.wait(10000, function() return predicate(state()) end, 20), label .. ": " .. vim.inspect(state()))
end

local function normal(s) assert(s.mode:sub(1, 1) == "n", "运行终端应保持普通模式: " .. s.mode) end

local ok, err = xpcall(function()
  vim.fn.rpcrequest(channel, "nvim_ui_attach", 100, 30, { rgb = true })
  assert(vim.wait(10000, function() return remote "return vim.v.vim_did_enter == 1" end, 20), "Neovim 未完成启动")
  local source_buf = remote([[
    vim.cmd.edit { args = { ... } }
    vim.bo.filetype = "sh"
    return vim.api.nvim_get_current_buf()
  ]], source)

  input " rr"
  wait_for("程序未进入输入等待", function(s) return s.visible and vim.tbl_contains(s.lines, "runner_ready") end)
  local started = state()
  normal(started)
  assert(started.current_buf == started.buf and started.exit == -1, "运行终端应已聚焦且程序仍在运行")

  input " rt"
  wait_for("Space rt 未隐藏终端", function(s) return not s.visible and s.current_buf == source_buf end)
  assert(state().exit == -1, "隐藏终端不应停止运行中的程序")

  input " rt"
  wait_for("Space rt 未重新显示终端", function(s) return s.visible and s.current_buf == started.buf end)
  local shown = state()
  normal(shown)
  assert(shown.job == started.job and vim.tbl_contains(shown.lines, "runner_ready"), "切换应保留同一进程和已有输出")

  input "ihello<CR>"
  wait_for("交互输入未被程序正确接收", function(s) return vim.tbl_contains(s.lines, "runner_input=hello") end)
  wait_for("程序未退出", function(s) return s.exit ~= -1 end)
  input "<C-\\><C-n>"
  wait_for("未返回普通模式", function(s) return s.mode:sub(1, 1) == "n" end)

  for _ = 1, 2 do
    input " rt"
    wait_for("程序结束后未隐藏终端", function(s) return not s.visible and s.current_buf == source_buf end)
    input " rt"
    wait_for("程序结束后未重新显示终端", function(s) return s.visible and s.current_buf == started.buf end)
    local reopened = state()
    normal(reopened)
    assert(vim.tbl_contains(reopened.lines, "runner_input=hello"), "程序结束后的输出不应丢失")
  end
  print "file_runner: normal-mode opening, live hide/show, stdin, and retained output passed"
end, debug.traceback)

vim.fn.jobstop(channel)
vim.fn.delete(root, "rf")
if not ok then
  io.stderr:write(err .. "\n")
  vim.cmd "cquit 1"
end
