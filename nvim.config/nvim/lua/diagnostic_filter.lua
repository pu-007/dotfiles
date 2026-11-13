-- LSP 诊断过滤引擎：在诊断写入 vim.diagnostic 之前应用 lua/diagnostic_ignores.lua 的规则。
-- push（publishDiagnostics）与 pull（textDocument/diagnostic）诊断都会经过 vim.diagnostic.set，
-- 在此处过滤可让虚拟文本、浮窗、诊断列表与跳转保持一致；只影响 LSP 客户端的命名空间，
-- 不影响其他插件写入的诊断。规则取消后，被隐藏的诊断可以从保存的原始数据恢复。
local M = { rules = {} }

-- bufnr -> namespace -> 被过滤前的完整诊断列表
local originals = {}
local orig_set, orig_reset

local function norm_severity(severity)
  if type(severity) == "number" then return severity end
  return vim.diagnostic.severity[(severity or "WARN"):upper()] or vim.diagnostic.severity.WARN
end

local function load_rules()
  package.loaded["diagnostic_ignores"] = nil
  local ok, rules = pcall(require, "diagnostic_ignores")
  if not ok or type(rules) ~= "table" then
    vim.notify("diagnostic_ignores.lua 加载失败：" .. tostring(rules), vim.log.levels.WARN)
    return {}
  end
  local normalized = {}
  for server, server_rules in pairs(rules) do
    normalized[server] = {}
    for _, rule in ipairs(server_rules) do
      table.insert(normalized[server], {
        severity = norm_severity(rule.severity),
        messages = rule.messages,
        codes = rule.codes,
        patterns = rule.patterns,
      })
    end
  end
  return normalized
end

-- LSP 诊断命名空间形如 nvim.lsp.<name>.<id>（push）或 nvim.lsp.<name>.<id>.<pull_id>（pull）
local function client_name_for_ns(namespace)
  local ns_name = vim.diagnostic.get_namespace(namespace).name
  for _, client in ipairs(vim.lsp.get_clients { _uninitialized = true }) do
    local prefix = ("nvim.lsp.%s.%d"):format(client.name, client.id)
    if ns_name == prefix or ns_name:sub(1, #prefix + 1) == prefix .. "." then return client.name end
  end
end

local function rule_matches(rule, diag)
  if diag.severity ~= rule.severity then return false end
  local listed = false
  if rule.messages then
    listed = true
    for _, message in ipairs(rule.messages) do
      if diag.message == message then return true end
    end
  end
  if rule.codes then
    listed = true
    for _, code in ipairs(rule.codes) do
      if tostring(diag.code) == tostring(code) then return true end
    end
  end
  if rule.patterns then
    listed = true
    for _, pattern in ipairs(rule.patterns) do
      if diag.message:find(pattern) then return true end
    end
  end
  return not listed
end

--- 过滤诊断列表并记录被隐藏的原始数据（用于规则取消后恢复）
function M.filter(namespace, bufnr, diagnostics)
  if next(M.rules) == nil then return diagnostics end
  local server = client_name_for_ns(namespace)
  local rules = server and M.rules[server]
  if not rules then return diagnostics end
  local kept, dropped = {}, false
  for _, diag in ipairs(diagnostics) do
    local ignored = false
    for _, rule in ipairs(rules) do
      if rule_matches(rule, diag) then
        ignored = true
        break
      end
    end
    if ignored then
      dropped = true
    else
      table.insert(kept, diag)
    end
  end
  originals[bufnr] = originals[bufnr] or {}
  originals[bufnr][namespace] = dropped and diagnostics or nil
  return kept
end

--- 重载规则并用当前规则重算已保存的原始诊断
function M.reload()
  M.rules = load_rules()
  for bufnr, by_ns in pairs(originals) do
    if vim.api.nvim_buf_is_valid(bufnr) then
      for namespace, diagnostics in pairs(by_ns) do
        orig_set(namespace, bufnr, M.filter(namespace, bufnr, diagnostics))
      end
    else
      originals[bufnr] = nil
    end
  end
end

function M.setup()
  if orig_set then return end
  M.rules = load_rules()
  orig_set, orig_reset = vim.diagnostic.set, vim.diagnostic.reset
  vim.diagnostic.set = function(namespace, bufnr, diagnostics, opts)
    if bufnr == 0 then bufnr = vim.api.nvim_get_current_buf() end
    return orig_set(namespace, bufnr, M.filter(namespace, bufnr, diagnostics), opts)
  end
  vim.diagnostic.reset = function(namespace, bufnr)
    if namespace == nil then
      originals = {}
    else
      for b in pairs(originals) do
        if bufnr == nil or b == bufnr then originals[b][namespace] = nil end
      end
    end
    return orig_reset(namespace, bufnr)
  end
  vim.api.nvim_create_user_command("LspIgnoreEdit", function()
    vim.cmd.edit(vim.fn.fnameescape(vim.fn.stdpath "config" .. "/lua/diagnostic_ignores.lua"))
  end, { desc = "编辑 LSP 诊断忽略规则" })
  vim.api.nvim_create_user_command("LspIgnoreReload", function() M.reload() end, { desc = "重载 LSP 诊断忽略规则" })
end

return M
