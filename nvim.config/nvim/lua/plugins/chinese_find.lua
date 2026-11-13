-- 中文标点等价的 f/F/t/T（基于 leap.nvim）。
-- 输入半角标点可同时命中对应全角标点（反之亦然），仅限当前行；
-- ; 与 , 仍为同向/反向重复，操作符（如 df,）与 . 重复可用（依赖 vim-repeat）。
---@type LazySpec
return {
  {
    url = "https://codeberg.org/andyg/leap.nvim",
    dependencies = { "tpope/vim-repeat" },
    config = function()
      local leap = require "leap"

      -- 互为别名的标点等价组：输入组内任一字符即可匹配组内全部字符，
      -- 需要增删等价关系时改这里即可。
      -- 注意：必须写入 leap 默认配置而非按调用传入——`.`(dot-repeat) 重放
      -- 不携带调用级 opts，只有默认层的等价组在重复时同样生效。
      -- 此设置会替换 leap 自带的等价组（空白 / 括号 / 引号），本配置没有
      -- 其他 leap 用法（未映射 s/S），如需恢复见 :h leap.opts。
      local equivalence_classes = {
        ",，",
        ";；",
        ":：",
        "!！",
        "?？",
        "(（",
        ")）",
        ".。",
        "\"“”",
        "'‘’",
      }
      leap.opts.equivalence_classes = equivalence_classes

      local function leap_opts()
        return {
          -- 直接跳到最近目标（保持原生 f 的手感），不显示标签
          labels = "",
          safe_labels = "",
          -- 遍历时 ; 前进、, 后退（与原生重复键一致）
          keys = { next_target = ";", prev_target = "," },
        }
      end

      local function find(kwargs)
        local was_op = vim.fn.mode(1):match "o" ~= nil
        local tick_before = vim.b.changedtick
        local seq_before = vim.g.repeat_sequence
        leap.leap(vim.tbl_extend("error", kwargs, {
          inputlen = 1,
          inclusive = true,
          -- 锁定当前行（官方建议：行号需在搜索开始前取值）
          pattern = function(pat) return "\\%" .. vim.fn.line "." .. "l" .. pat end,
          opts = leap_opts(),
        }))
        -- vim-repeat 的 repeat#set 记录的是变更前 changedtick，原本依赖随后的
        -- CursorMoved 补齐；但 df, 等操作完成后光标可能停在原位，且操作符的
        -- 删除发生在本函数返回之后，故延迟到事件循环同步，保证操作后立即
        -- 按 . 也能重放
        if was_op and vim.g.repeat_sequence ~= seq_before then
          local bufnr = vim.api.nvim_get_current_buf()
          vim.schedule(function()
            if vim.api.nvim_buf_is_valid(bufnr) and vim.b[bufnr].changedtick ~= tick_before then
              vim.g.repeat_tick = vim.b[bufnr].changedtick
            end
          end)
        end
      end

      local function repeat_find(invert)
        local rep = leap.state["repeat"]
        if not rep.in1 then return end
        -- leap 保存的 backward 为 nil/false（向前）或 true（向后），先归一化再按需取反
        local backward = rep.backward == true
        if invert then backward = not backward end
        leap.leap { ["repeat"] = true, backward = backward, opts = leap_opts() }
      end

      local modes = { "n", "x", "o" }
      vim.keymap.set(modes, "f", function() find {} end, { desc = "查找下一个字符（中英标点等价）" })
      vim.keymap.set(modes, "F", function() find { backward = true } end, { desc = "查找上一个字符（中英标点等价）" })
      vim.keymap.set(modes, "t", function() find { offset = -1 } end, { desc = "移到下一个字符之前（中英标点等价）" })
      vim.keymap.set(modes, "T", function() find { backward = true, offset = 1 } end, { desc = "移到上一个字符之后（中英标点等价）" })
      vim.keymap.set(modes, ";", function() repeat_find(false) end, { desc = "重复字符查找" })
      vim.keymap.set(modes, ",", function() repeat_find(true) end, { desc = "反向重复字符查找" })
    end,
  },
}
