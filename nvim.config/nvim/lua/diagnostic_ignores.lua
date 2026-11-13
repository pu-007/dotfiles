-- LSP 诊断忽略规则：按语言服务器名分组，方便批量添加。
-- 字段说明（同一规则内 messages/codes/patterns 为“或”关系，服务器 + severity 为“与”关系）：
--   severity: "ERROR" | "WARN" | "INFO" | "HINT"，默认 "WARN"
--   messages: 完整匹配的诊断消息列表
--   codes:    诊断码列表（诊断的 code 字段）
--   patterns: Lua 模式列表（建议用 [[...]] 书写）；三者都不填时忽略该服务器此级别的全部诊断
-- 修改后执行 :LspIgnoreReload 立即生效；:LspIgnoreEdit 打开本文件。
return {
  marksman = {
    {
      severity = "WARN",
      patterns = {
        [[^Ambiguous link to document ']],
      },
    },
  },

  -- 更多示例（按需取消注释；取消注释后对应诊断即被忽略）：
  -- marksman = {
  --   { severity = "WARN", messages = { "Ambiguous link to document 'docs/00_roadmap.md'" } },
  --   { severity = "WARN", patterns = { [[^Ambiguous link]], [[^Unknown scheme]] } },
  -- },
  -- lua_ls = {
  --   { severity = "WARN", codes = { "trailing-space", "assign-type-mismatch" } },
  -- },
  -- basedpyright = {
  --   { severity = "INFO", messages = { "Import \"foo\" could not be resolved" } },
  -- },
  -- ruff = {
  --   { severity = "WARN" }, -- 忽略 ruff 的全部警告
  -- },
}
