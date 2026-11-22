-- 从配置根目录运行：nvim --headless -u NONE -i NONE -l tests/visual_motion.lua
-- 加载真实配置并输入真实按键，验证普通 / 逐行可视 / 块选择模式下的 j、k 与计数。
local channel = vim.fn.jobstart({ vim.v.progpath, "--embed", "--headless", "-i", "NONE" }, { rpc = true })
assert(channel > 0, "无法启动 Neovim 子进程")

local function remote(code) return vim.fn.rpcrequest(channel, "nvim_exec_lua", code, {}) end
local function input(keys) vim.fn.rpcrequest(channel, "nvim_input", keys) end

local function state()
  return remote [[
    local pos = vim.api.nvim_win_get_cursor(0)
    return { mode = vim.fn.mode(), row = pos[1], col = pos[2], from = vim.fn.getpos("v")[2] }
  ]]
end

-- 按键由子进程异步处理，轮询到期望状态；映射失效时光标不动，超时后报告实际状态。
local function expect(label, want)
  local got
  local reached = vim.wait(3000, function()
    got = state()
    for key, value in pairs(want) do
      if got[key] ~= value then return false end
    end
    return true
  end, 20)
  assert(reached, ("%s: 期望 %s，实际 %s"):format(label, vim.inspect(want), vim.inspect(got)))
end

local function reset()
  input "<Esc>"
  expect("回到普通模式", { mode = "n" })
  remote [[
    vim.api.nvim_buf_set_lines(0, 0, -1, false, { "aaaa", "bbbb", "cccc", "dddd", "eeee" })
    vim.api.nvim_win_set_cursor(0, { 1, 1 })
  ]]
end

local ok, err = xpcall(function()
  vim.fn.rpcrequest(channel, "nvim_ui_attach", 100, 30, { rgb = true })
  assert(vim.wait(20000, function() return remote "return vim.v.vim_did_enter == 1" end, 50), "Neovim 未完成启动")
  remote [[ vim.cmd.enew { bang = true } ]]
  reset()

  input "j"
  expect("普通模式 j", { mode = "n", row = 2 })
  input "2j"
  expect("普通模式 2j", { row = 4 })
  input "k"
  expect("普通模式 k", { row = 3 })

  reset()
  input "V"
  expect("V", { mode = "V", row = 1, from = 1 })
  input "j"
  expect("V 后 j", { mode = "V", row = 2, from = 1 })
  input "2j"
  expect("V 后 2j", { mode = "V", row = 4, from = 1 })
  input "k"
  expect("V 后 k", { mode = "V", row = 3, from = 1 })

  reset()
  input "<C-v>"
  expect("Ctrl+V", { mode = "\22", row = 1, col = 1 })
  input "j"
  expect("Ctrl+V 后 j", { mode = "\22", row = 2, col = 1, from = 1 })
  input "l"
  expect("Ctrl+V 后 l", { mode = "\22", row = 2, col = 2, from = 1 })
  input "2j"
  expect("Ctrl+V 后 2j", { mode = "\22", row = 4, col = 2, from = 1 })
  input "k"
  expect("Ctrl+V 后 k", { mode = "\22", row = 3, col = 2, from = 1 })

  reset()
  print "visual_motion: normal j/k, linewise V and blockwise Ctrl+V extension passed"
end, debug.traceback)

vim.fn.jobstop(channel)
if not ok then
  io.stderr:write(err .. "\n")
  vim.cmd "cquit 1"
end
