-- AstroNvim 适配层；算法和默认标点映射表由 pu-007/chinese_find.nvim 提供。
---@type LazySpec
return {
  "AstroNvim/astrocore",
  dependencies = { "pu-007/chinese_find.nvim" },
  opts = function(_, opts)
    local find = require "chinese_find"
    find.setup { mappings = false }
    for _, mode in ipairs { "n", "x", "o" } do
      opts.mappings[mode] = opts.mappings[mode] or {}
      for _, key in ipairs { "f", "F", "t", "T", ";", "," } do
        opts.mappings[mode][key] = {
          function() return find.expr(key) end,
          expr = true,
          desc = "字符查找 " .. key .. "（中英标点等价）",
        }
      end
    end
  end,
}
