-- 从配置根目录运行：nvim --headless -u NONE -i NONE -l tests/chinese_find.lua
-- 两个独立进程接收真实按键：当前配置与原生 Vim 语义对照，不调用插件内部状态。
local standalone = arg[1] == "standalone"
local surrounds_only = arg[1] == "surround"
local children = {}
local function child(configured)
  local args = { vim.v.progpath, "--embed", "--headless", "-i", "NONE" }
  if not configured or standalone then vim.list_extend(args, { "-u", "NONE" }) end
  if configured and standalone then
    local plugin_dir = vim.fn.stdpath "data" .. "/lazy/chinese_find.nvim"
    vim.list_extend(args, {
      "--cmd", "lua vim.opt.runtimepath = { vim.env.VIMRUNTIME, " .. vim.inspect(plugin_dir) .. " }; vim.opt.packpath = {}",
      "-c", "lua require('chinese_find').setup()",
    })
  end
  local channel = vim.fn.jobstart(args, { rpc = true })
  assert(channel > 0, "无法启动 Neovim 子进程")
  children[#children + 1] = channel
  vim.fn.rpcrequest(channel, "nvim_ui_attach", 100, 30, { rgb = true })
  return function(code, ...)
    return vim.fn.rpcrequest(channel, "nvim_exec_lua", code, { ... })
  end, channel
end

local function settle()
  -- 让被测进程完成按键、CursorMoved 和 schedule 回调，再采集结果。
  vim.wait(30, function() return false end, 10)
end

local function input(channel, keys)
  vim.fn.rpcrequest(channel, "nvim_input", keys)
  settle()
end

local function prepare(remote, lines, col)
  remote([[
    local lines, col = ...
    vim.cmd.enew { bang = true }
    vim.bo.swapfile = false
    vim.bo.undolevels = -1
    vim.api.nvim_buf_set_lines(0, 0, -1, false, lines)
    vim.bo.undolevels = 1000
    vim.api.nvim_win_set_cursor(0, { 1, vim.fn.byteidx(lines[1], col or 0) })
  ]], lines, col)
end

local function snapshot(remote)
  return remote [[
    local pos = vim.api.nvim_win_get_cursor(0)
    return {
      lines = vim.api.nvim_buf_get_lines(0, 0, -1, false),
      row = pos[1],
      col = vim.fn.strchars(vim.api.nvim_get_current_line():sub(1, pos[2])),
      mode = vim.fn.mode(),
      register = vim.fn.getreg('"'),
    }
  ]]
end

local function normalize(value)
  if type(value) == "table" then
    for k, v in pairs(value) do value[k] = normalize(v) end
  elseif type(value) == "string" then
    for _, pair in ipairs { { "，", "," }, { "；", ";" }, { "。", "." }, { "（", "(" }, { "）", ")" }, { "“", '"' }, { "”", '"' }, { "、", "\\" } } do
      value = value:gsub(pair[1], function() return pair[2] end)
    end
  end
  return value
end

local ok, err = xpcall(function()
  local actual, actual_channel = child(true)
  local native, native_channel = child(false)
  assert(vim.wait(10000, function() return actual "return package.loaded['chinese_find'] ~= nil" end, 50), "模块未加载")
  local cases = {
    { "f 后 ;/, 不改变原方向", { "a,b，c,d,e" }, 0, { "f,", ";", ",", ";", "2;" } },
    { "F 后 ;/, 不改变原方向", { "a,b，c,d,e" }, 8, { "F,", ";", ",", ";" } },
    { "t 连续重复与反向偏移", { "ab,cd，ef,gh,ij" }, 0, { "t,", ";", ";", ",", ",", ";" } },
    { "T 连续重复与反向偏移", { "ab,cd，ef,gh,ij" }, 13, { "T,", ";", ";", ",", ";" } },
    { "t 邻接目标可原地停止", { "a，b,c" }, 0, { "t,", ";" } },
    { "T 邻接目标可原地停止", { "a,b，c" }, 4, { "T,", ";" } },
    { "计数与新目标", { "a,b，c;d；e" }, 0, { "2f,", "f;", "f;", "F，", "t;" } },
    { "全角分号查找", { "a;b；c;d" }, 0, { "f；", ";", "," } },
    { "连续 df 和点重复", { "a,b，c,d,e" }, 0, { "df,", ".", "." } },
    { "dt 和点重复", { "ab,cd，ef,gh" }, 0, { "dt,", ".", "." } },
    { "dT 和点重复", { "ab,cd，ef,gh" }, 10, { "dT,", "." } },
    { "操作符计数相乘", { "a,b，c,d，e,f" }, 0, { "2d2f," } },
    { "字符查找后 d;", { "a,b，c,d" }, 0, { "f,", "d;", "." } },
    { "修改后点重复", { "a,b，c,d" }, 0, { "cf,X<Esc>", "l", "." } },
    { "可视选择删除", { "a,b，c,d" }, 0, { "vf,", ";", "d" } },
    { "反向可视 till", { "ab,cd，ef,gh" }, 10, { "vT,", ";", "d" } },
    { "复制到目标", { "a,b，c,d" }, 0, { "yf," } },
    { "不能跨行", { "abc", "d,e" }, 0, { "f,", "df," } },
    { "换行后重复查找", { "a,b", "c，d,e" }, 0, { "f,", "j0", ";" } },
    { "换行后点重复", { "a,b", "c，d,e" }, 0, { "df,", "j0", "." } },
    { "不足计数不删除", { "a，b" }, 0, { "3df," } },
    { "大小写敏感", { "xAaAa" }, 0, { "fa", ";", "FA" } },
    { "正则字符按字面", { "a[b[c\\d\\e" }, 0, { "f[", ";", "f\\", ";" } },
    { "句号与引号等价", { 'a.b。c“d”e(f）g' }, 0, { "f.", ";", 'f"', ";", "f(", "f)" } },
    { "目标为 f/t", { "aftft" }, 0, { "ff", "ft", "ff", "ft" } },
    { "取消查找", { "a，b,c" }, 0, { "f<Esc>", "t<Esc>" } },
    { "普通修改接管点重复", { "a,b，c,d" }, 0, { "df,", "x", "." } },
    { "宏录制与回放", { "a,b，c,d" }, 0, { "qqdf,q", "@q", "@@" } },
    { "s 保持替换字符", { "abc" }, 0, { "sX<Esc>" } },
    { "顿号与反斜杠双向等价", { "甲、乙\\丙、丁" }, 0, { "f\\", ";", "F、", "f、" } },
    { "中文字符两侧 till 偏移", { "甲，乙,丙；丁" }, 0, { "t,", ";", "T,", "t;" } },
    { "顿号删除与点重复", { "甲、乙\\丙、丁" }, 0, { "df\\", ".", "." } },
    { "带计数的 till 重复", { "ab,cd，ef,gh,ij" }, 0, { "t,", "2;", "2,", ";" } },
    { "点重复指定次数", { "a,b，c,d，e,f,g" }, 0, { "df,", "2." } },
    { "点重复保留原计数", { "a,b，c,d，e,f,g" }, 0, { "2df,", "." } },
    { "强制字符运动", { "a,b，c,d" }, 0, { "dvf," } },
    { "强制行运动", { "a,b，c,d", "next" }, 0, { "dVf," } },
    { "反向查找后操作符反向重复", { "a,b，c,d" }, 6, { "F,", "d," } },
    { "重复操作使用最新查找", { "a,b，c;d,e;f" }, 0, { "f,", "d;", "f;", "0", "." } },
    { "df 全角分号连续重复", { "甲；乙;丙；丁;戊" }, 0, { "df；", ".", "." } },
    { "dt 全角分号重复", { "甲乙；丙丁;戊己；庚" }, 0, { "dt；", ".", "." } },
    { "全角分号操作符计数", { "a；b;c；d;e；f" }, 0, { "d2f；", "." } },
    { "全角分号查找后 d;", { "a；b;c；d;e" }, 0, { "f；", "d;", "." } },
    { "强制 till 邻接目标不删除", { "a，b,c" }, 0, { "dvt," } },
  }
  if not surrounds_only then
    local failures, checks = {}, 0
    for _, case in ipairs(cases) do
      input(actual_channel, "<Esc>")
      input(native_channel, "<Esc>")
      prepare(actual, case[2], case[3])
      prepare(native, normalize(vim.deepcopy(case[2])), case[3])
      actual "vim.fn.setreg('\"', '')"
      native "vim.fn.setreg('\"', '')"
      for _, keys in ipairs(case[4]) do
        input(actual_channel, keys)
        input(native_channel, normalize(keys))
        local got, expected = normalize(snapshot(actual)), snapshot(native)
        checks = checks + 1
        if not vim.deep_equal(got, expected) then
          failures[#failures + 1] = case[1] .. " / " .. keys .. "\n实际: " .. vim.inspect(got) .. "\n预期: " .. vim.inspect(expected)
          break
        end
      end
    end
    print(("motion scenarios=%d assertions=%d failures=%d"):format(#cases, checks, #failures))
    assert(#failures == 0, table.concat(failures, "\n"))
  end
  if not standalone then
    assert(vim.wait(10000, function() return actual "return package.loaded['nvim-surround'] ~= nil" end, 50), "Surround 未加载")
    local surrounds = {
      { "添加双引号", { "hello" }, { 'ysiw"' }, { '"hello"' } },
      { "删除双引号", { '"hello"' }, { 'ds"' }, { "hello" } },
      { "双引号改成单引号", { '"hello"' }, { "cs\"'" }, { "'hello'" } },
      { "括号替换", { "(hello)" }, { "cs)]" }, { "[hello]" } },
      { "左括号添加空格", { "hello" }, { "ysiw(" }, { "( hello )" } },
      { "可视包围", { "hello world" }, { 'veS"' }, { '"hello" world' } },
      { "包围操作与中文 till", { "甲乙；丙" }, { "yst；)" }, { "(甲乙)；丙" } },
      { "包围操作点重复", { "one", "two" }, { 'yss"', "j", "." }, { '"one"', '"two"' } },
      { "中文词可视包围", { "你好 世界" }, { 'viwS"' }, { '"你好" 世界' } },
      { "Jieba 修改词及点重复", { "hello world" }, { "ciw", "X<Esc>", "w", "." }, { "X X" } },
    }
    for _, case in ipairs(surrounds) do
      print("surround: " .. case[1])
      input(actual_channel, "<Esc>")
      prepare(actual, case[2], 0)
      for _, keys in ipairs(case[3]) do input(actual_channel, keys) end
      local got = snapshot(actual).lines
      assert(vim.deep_equal(got, case[4]), case[1] .. ": " .. vim.inspect(got))
    end
    print(("surround/Jieba scenarios=%d failures=0"):format(#surrounds))
  end
end, debug.traceback)

for _, channel in ipairs(children) do vim.fn.jobstop(channel) end
if not ok then
  io.stderr:write(err .. "\n")
  vim.cmd "cquit 1"
end
