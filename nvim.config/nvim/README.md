# AstroNvim 配置与操作手册

基于 AstroNvim v6。语言包和通用插件优先从 AstroCommunity 导入，通用按键通过 AstroCore 的 `opts.mappings` 配置；中文标点查找使用独立插件 [chinese_find.nvim](https://github.com/pu-007/chinese_find.nvim)，诊断过滤和单文件运行保留为独立 Lua 模块。

## 按键约定与配置入口

- `<Leader>` 是空格。下文 `Space ff` 表示依次按空格、`f`、`f`。
- `<LocalLeader>` 是反斜杠 `\`，不再占用反向查找键 `,`。
- `Ctrl+x` 表示同时按键；`Esc` 返回普通模式。下文未注明模式的操作均在普通模式使用。
- 本配置的 `H` / `L` 已改为行首非空白字符 / 行尾，不是 Vim 默认的屏幕上方 / 下方。
- 横向分屏从单独的 `\` 移到 `Space \`，避免和 `<LocalLeader>` 共用前缀；纵向分屏仍用 `|`。

按空格后稍等可以查看 which-key 提示；`Space fk` 搜索当前快捷键，`Space fa` 查找配置文件。LSP 快捷键只在对应语言服务器支持该能力时出现。

## 配置结构

加载顺序：`init.lua` 引导 lazy.nvim，`lua/lazy_setup.lua` 依次导入 AstroNvim 默认配置、`community`、`plugins`，最后执行 `lua/polish.lua`。

| 文件                            | 职责 / 适合修改的内容                                             |
| ------------------------------- | ----------------------------------------------------------------- |
| `init.lua`                      | lazy.nvim 引导入口，通常无需改动                                  |
| `lua/lazy_setup.lua`            | AstroNvim 大版本、Leader / LocalLeader、插件导入顺序              |
| `lua/community.lua`             | AstroCommunity 插件及语言包的统一入口                             |
| `lua/plugins/astrocore.lua`     | 通用按键：H/L、加速 j/k、分屏、运行菜单分组                       |
| `lua/plugins/astroui.lua`       | Gruvbox 主题、图标和高亮                                          |
| `lua/plugins/appearance.lua`    | Dashboard、Presence 等外观配置                                    |
| `lua/plugins/editing.lua`       | 加速移动、LuaSnip、LaTeX `$` 配对                                 |
| `lua/plugins/chinese_input.lua` | jieba 中文分词、输入法切换，以及 jieba 的点重复依赖               |
| `lua/plugins/chinese_find.lua`  | chinese_find.nvim 仓库依赖、选项和 AstroCore 映射适配             |
| `lua/diagnostic_ignores.lua`    | 日常修改的 LSP 警告忽略规则                                       |
| `lua/diagnostic_filter.lua`     | 诊断过滤、缓存、规则重载引擎                                      |
| `lua/plugins/code_runner.lua`   | code_runner 插件及运行快捷键                                      |
| `lua/file_runner.lua`           | 各语言执行命令、Python 解释器选择、运行终端生命周期               |
| `lua/polish.lua`                | 初始化诊断过滤                                                    |
| `lazy-lock.json`                | 锁定插件版本，随配置一起保存                                      |
| `tests/chinese_find.lua`        | 原生 Vim 对照回归；完整配置下还验证 surround 与中文 motion 的配合 |

新增社区插件时，先在 `community.lua` 中添加 `{ import = "astrocommunity.分类.插件名" }`。需要调整选项，再在 `lua/plugins/` 中用同一插件名的 `opts` 覆盖；无需复制社区配方，也无需再次手动调用插件的 `setup()`。

普通选项和通用映射写进 `AstroNvim/astrocore` 的 `opts.options`、`opts.mappings`；LSP 定制使用 `AstroNvim/astrolsp` 的 `opts`。当前语言包已在 `community.lua` 中按语言列出，安装服务器、格式化器和解析器优先沿用相应 pack。

## 中文标点等价查找

### f/F/t/T 与重复

查找只在当前行进行，区分大小写；普通字符按字面匹配。等价仅作用于查找目标，不会替换文件里的标点，也不会修改 `/` 搜索模式。

| 按键          | 行为                                    |
| ------------- | --------------------------------------- |
| `f,` 或 `f，` | 向右找到最近的 `,` 或 `，`，停在标点上  |
| `F,`          | 向左找逗号，停在标点上                  |
| `t,`          | 向右找逗号，停在它前一个字符            |
| `T,`          | 向左找逗号，停在它后一个字符            |
| `f;` 或 `f；` | 向右查找半角或全角分号                  |
| `f\` 或 `f、` | 向右查找反斜杠或顿号                    |
| `2f,`         | 找第 2 个等价逗号                       |
| `;`           | 按最近一次 f/F/t/T 的原方向重复         |
| `,`           | 临时反向重复；不会改变后续 `;` 的原方向 |
| `2;` / `2,`   | 带计数的同向 / 反向重复                 |
| `Esc`         | 取消正在等待目标字符的查找              |

每次按 `f/F/t/T` 都读取新目标，不会因上一次查找仍处于活动状态而吞掉新输入。单独重复查找的命令键是半角 `;`、`,`；作为目标的字符则可以输入全角或半角。

`f\` 中的反斜杠是 f 正在读取的目标字符，不会触发 `<LocalLeader>`。这与普通模式单独按 `\` 是两个不同场景。

### 与操作符、点重复配合

| 按键              | 行为                                                  |
| ----------------- | ----------------------------------------------------- |
| `df,`             | 删除到下一个逗号，包含标点                            |
| `dt,`             | 删除到下一个逗号之前，保留标点                        |
| `df；`            | 删除到下一个 `;` 或 `；`，包含标点                    |
| `dt；`            | 删除到下一个分号之前                                  |
| `d2f；` / `2df；` | 删除到第 2 个等价分号                                 |
| `df\`             | 删除到下一个反斜杠或顿号                              |
| `cf；`            | 修改到分号并进入插入模式                              |
| `yf；`            | 复制到分号                                            |
| `vf；`            | 可视选择到分号                                        |
| `d;` / `d,`       | 以最近的字符查找为 motion 删除                        |
| `.`               | 重复上一次修改，包括 `df；`、`cf；` 等                |
| `2.`              | 用新计数 2 重复上一次修改；不是简单执行两次独立的 `.` |

例如光标在 `甲；乙;丙；丁` 的 `甲` 上，执行 `df；` 后得到 `乙;丙；丁`，紧接着按 `.` 得到 `丙；丁`，再按 `.` 得到 `丁`。删除后即使光标没移动，也可以立即重复。

`t/T` 在第一次查找相邻标点时可以停在原位；随后 `;` 会继续寻找下一处目标。无目标或计数不足时不越过行边界，也不会误删当前字符。模块保留了计数、宏、普通 / 可视 / 操作符待决模式和原生点重复。

### 映射表与独立复用

默认映射表位于 [chinese_find.nvim 的 lua/chinese_find.lua](https://github.com/pu-007/chinese_find.nvim/blob/main/lua/chinese_find.lua) 的 `defaults`，可通过 `require("chinese_find").defaults` 读取：

```lua
{
  [","] = "，",
  [";"] = "；",
  [":"] = "：",
  ["!"] = "！",
  ["?"] = "？",
  ["("] = "（",
  [")"] = "）",
  ["."] = "。",
  ['"'] = "“”",
  ["'"] = "‘’",
  ["\\"] = "、",
}
```

每个键及其值里的所有字符组成一个**双向等价组**。例如输入 `“`、`”` 或 `"`，都能找到该组内的任意字符。不同组不要重复包含同一个字符。Lua 字符串里的 `"\\"` 表示一个实际的反斜杠。

其他 Neovim 用户可直接用 lazy.nvim 安装，无需 AstroNvim、Leap、Flash 或 vim-repeat：

```lua
{
  "pu-007/chinese_find.nvim",
  main = "chinese_find",
  event = "VeryLazy",
  opts = {},
}
```

建议在插件加载前设置 `vim.g.maplocalleader = "\\"`，避免逗号前缀延迟；同时协调其他占用该前缀的映射。仍可从插件仓库单独复制 `lua/chinese_find.lua`，再执行 `require("chinese_find").setup()`。

自定义时可替换整张等价表：

```lua
require("chinese_find").setup {
  equivalents = {
    [","] = "，",
    [";"] = "；",
    ["\\"] = "、",
  },
}
```

没有列出的字符仍按字面查找。本配置由 `lua/plugins/chinese_find.lua` 声明仓库依赖，调用 `setup { mappings = false }`，再用 AstroCore 注册六个表达式映射。定制规则时，把 `equivalents` 传给适配层已有的 `setup` 调用；如需保留默认表，可先 `vim.deepcopy(find.defaults)` 再增改。不要修改插件安装目录中的源码，也不要在本配置的 `lua/` 中保留同名模块，以免遮蔽插件更新。

### 中文按词移动与输入法

jieba.vim 增强 `w/W`、`b/B`、`e/E`、`ge/gE` 和 `iw/aw` 等文本对象，适合中文词语移动、`ciw` 修改词、`diw` 删除词。输入法切换由 `im-select-ahk.nvim` 提供：普通模式切英文，插入和命令行模式恢复输入法；该集成依赖实际的 Windows / AHK 输入法环境。

词语按键由 AstroCore 统一注册。`g@` 自定义操作符（surround 使用它）走 Vim 原生的词范围，以免 jieba 在执行操作符时附带的 `Esc` 取消后续输入；普通移动、`ciw/diw/yiw` 继续使用 jieba。需要精确包围一个中文词时，先用 `viw` 按 jieba 分词选中，再按 `S"`。

## Surround：添加、删除、替换引号和括号

使用社区配方 `astrocommunity.motion.nvim-surround`，保留插件默认按键。光标放在待操作文字或包围符内部即可。

记法：`ys{范围}{新符号}` 添加，`ds{旧符号}` 删除，`cs{旧符号}{新符号}` 替换。

| 原文 / 操作范围        | 按键     | 结果                             |
| ---------------------- | -------- | -------------------------------- |
| `hello`，光标在词上    | `ysiw"`  | `"hello"`                        |
| `hello`                | `ysiw'`  | `'hello'`                        |
| `hello`                | `ysiw)`  | `(hello)`                        |
| `hello`                | `ysiw(`  | `( hello )`，带内部空格          |
| 当前整行               | `yss"`   | 给整行内容加双引号               |
| 从光标到行尾           | `ys$]`   | 用方括号包围该范围               |
| `"hello"`              | `ds"`    | `hello`                          |
| `(hello)`              | `ds)`    | `hello`                          |
| `( hello )`            | `ds(`    | `hello`，同时去除贴近括号的空格  |
| `"hello"`              | `cs"'`   | `'hello'`                        |
| `'hello'`              | `cs'"`   | `"hello"`                        |
| `(hello)`              | `cs)]`   | `[hello]`                        |
| `[hello]`              | `cs]}`   | `{hello}`                        |
| 先 `viw` 选择一个词    | `S"`     | 给选区加双引号                   |
| `甲乙；丙`，光标在甲上 | `yst；)` | `(甲乙)；丙`，可组合中文等价查找 |

括号输入**右括号**时不加空格，输入**左括号**时添加内部空格，`() / [] / {} / <>` 都遵循这个约定。普通模式 `s` 仍是替换当前字符；只有可视模式的 `S` 用于 surround。

其他常用操作：

- 插入模式按 `Ctrl+g` 再按 `s`、`"`，插入一对双引号。
- `ysiw` 后按反引号，可给当前词添加 Markdown 行内代码标记。
- `ysiw` 后按 `t`，输入 `strong` 并回车，可添加 `<strong>...</strong>`；`dst` 删除外围 HTML 标签。
- `dsq` 删除最近的引号对；`q` 是单双引号和反引号的查找别名，不用于添加。
- `.` 重复上次 surround 修改，`u` 撤销。
- 这里的默认 surround 处理英文引号和常规括号；字符查找的中英等价表不会自动扩展 surround 的包围符定义。

## 文件、窗口、搜索

| 按键                    | 功能                                      |
| ----------------------- | ----------------------------------------- |
| `Space w`               | 保存文件                                  |
| `Space q` / `Space Q`   | 关闭窗口 / 退出 AstroNvim，必要时询问保存 |
| `Space n`               | 新建缓冲区                                |
| `Space c` / `Space bc`  | 关闭当前缓冲区 / 关闭其他缓冲区           |
| `]b` / `[b`             | 下一个 / 上一个缓冲区                     |
| `Space e` / `Space o`   | 显示隐藏文件树 / 切换文件树焦点           |
| `Space ff`              | 查找文件                                  |
| `Space fF`              | 查找包括隐藏、忽略文件在内的文件          |
| `Space fw`              | 项目内搜索文本，需要 ripgrep              |
| `Space fc`              | 搜索光标下的词                            |
| `Space fb` / `Space fo` | 查找缓冲区 / 最近文件                     |
| `Space f<Enter>`        | 恢复上一次搜索面板                        |
| `Space fk` / `Space fh` | 搜索快捷键 / 帮助                         |
| `Space \` / `\|`        | 横向 / 纵向分屏                           |
| `Ctrl+h/j/k/l`          | 在分屏之间移动                            |
| `Ctrl+方向键`           | 调整分屏尺寸                              |
| `Space /`               | 注释当前行或可视选区                      |

## LSP、补全和诊断过滤

| 按键 / 命令             | 功能                                        |
| ----------------------- | ------------------------------------------- |
| `K`                     | 查看悬浮说明                                |
| `gd` / `gD`             | 跳到定义 / 声明                             |
| `gI` / `gy`             | 跳到实现 / 类型定义                         |
| `Space lR` / `Space lr` | 查找引用 / 重命名符号                       |
| `Space la` / `Space lf` | 代码操作 / 格式化                           |
| `Space uf` / `Space uF` | 当前缓冲区 / 全局保存时自动格式化开关       |
| `Space ld` / `Space lD` | 当前诊断浮窗 / 诊断搜索面板                 |
| `[e` / `]e`             | 上一个 / 下一个错误                         |
| `[w` / `]w`             | 上一个 / 下一个警告                         |
| `Space lv`              | 选择 Python 虚拟环境，需 venv-selector 可用 |
| `:LspIgnoreEdit`        | 打开诊断忽略规则                            |
| `:LspIgnoreReload`      | 重载规则，取消的规则所隐藏的诊断也会恢复    |

插入模式补全：`Ctrl+Space` 打开菜单，`Ctrl+n/p` 或 `Ctrl+j/k` 选择，`Enter` 接受，`Ctrl+e` 关闭；`Tab` 优先选下一个候选，其次跳到下一个 snippet 占位符。

日常只修改 `lua/diagnostic_ignores.lua`。下面表示仅忽略 marksman 的指定 WARN；在同一个服务器列表里可放多条规则，不要重复定义同名 `marksman` 键：

```lua
return {
  marksman = {
    {
      severity = "WARN",
      patterns = { [[^Ambiguous link to document ']] },
    },
    -- 只忽略一条完整消息也可以：
    -- { severity = "WARN", messages = { "Ambiguous link to document 'docs/00_roadmap.md'" } },
  },
  -- lua_ls = {
  --   { severity = "WARN", codes = { "trailing-space", "assign-type-mismatch" } },
  -- },
}
```

`severity` 可为 `ERROR/WARN/INFO/HINT`，默认 `WARN`。`messages` 完整匹配，`codes` 匹配诊断码，`patterns` 使用 **Lua 模式**；同一规则内这三种匹配为“或”。如果三个字段都不填，会忽略该服务器对应级别的全部诊断。过滤覆盖 LSP push / pull 诊断，其他插件的非 LSP 诊断不受影响。

## 单文件运行与终端

- `Space rr`：先保存当前文件，然后执行；重跑时停止旧运行进程、替换旧终端。打开终端后保持普通模式，方便查看输出和使用快捷键。
- `Space rt`：在普通模式下显示 / 隐藏最近的运行终端，重新显示也保持普通模式；隐藏不会停止程序，程序结束后保留输出。
- 运行终端中需要输入时按 `i`；输入模式下按键会发送给程序，先按 `Ctrl+\` 再按 `Ctrl+n` 回到普通模式，才能使用 `Space rt`。
- 普通 shell 使用 `Space tf`（浮动）、`Space th`（横向）、`Space tv`（纵向），`F7` 切换 ToggleTerm。

| 文件类型   | 执行方式                                                                        |
| ---------- | ------------------------------------------------------------------------------- |
| Python     | 优先使用 venv-selector 选择的解释器，否则 PATH 中的 python3 / python；使用 `-u` |
| C / C++    | gcc / g++ 编译到临时文件，成功后运行                                            |
| Rust       | rustc 编译单文件，成功后运行                                                    |
| JavaScript | node                                                                            |
| sh / bash  | bash                                                                            |
| Lua        | PATH 中的 lua / luajit                                                          |

工作目录为当前源文件所在目录。解释器、编译器需要已安装。TypeScript 尚未指定运行时，可在 `file_runner.lua` 中按项目选择 tsx 或 Deno 示例。工程构建仍应使用项目自己的构建命令。

`:RunFile`、`:RunCode` 等是 code_runner 原生命令，使用插件自己的终端路径，不共享 `Space rt` 管理的终端。日常运行当前文件优先使用 `Space rr`。

## 常用 Neovim 编辑技巧

Vim 操作可以按“操作符 + 范围”组合：`d` 删除、`c` 修改、`y` 复制；`i` 文本对象只含内部，`a` 连同外围。例如 `ci"` 修改引号内容，`da)` 删除括号及其内容，`yiw` 复制一个词。Surround 的 `ds)` 只删除括号，两者作用不同。

| 操作                   | 用法                                                           |
| ---------------------- | -------------------------------------------------------------- |
| 快速改词               | `ciw` 输入新词，`Esc` 退出；移动到下一个词后用 `.` 重复        |
| 修改引号内容           | `ci"` 保留引号，替换里面的内容                                 |
| 删除到行尾             | `D`；修改到行尾用 `C`                                          |
| 插入位置               | `I` 到行首插入，`A` 到行尾追加，`o/O` 在下方 / 上方新建一行    |
| 定位                   | `gg` 文件开头，`G` 文件结尾，`42G` 到第 42 行，`%` 找配对括号  |
| 原生文本搜索           | `/内容` 向下，`?内容` 向上，`n/N` 重复；`*` 搜索光标下的词     |
| 撤销 / 重做            | `u` / `Ctrl+r`；`Space fu` 查看撤销历史                        |
| 返回跳转位置           | `Ctrl+o` 返回，`Ctrl+i` 前进；适合查定义后回到调用处           |
| 返回编辑位置           | `g;` / `g,` 在修改记录中移动，与裸 `;` / `,` 的字符查找不同    |
| 重选上一选区           | `gv`                                                           |
| 选区缩进               | 可视模式 `Tab` / `Shift+Tab`，操作后保持选区                   |
| 矩形编辑               | `Ctrl+v` 选矩形，`I` 输入，`Esc` 后应用到各行                  |
| 不覆盖复制寄存器地删除 | `"_d{范围}`，例如 `"_diw`                                      |
| 系统剪贴板             | `"+y` 复制选区，`"+p` 粘贴；需要系统剪贴板 provider            |
| 录制宏                 | `qa` 开始录到 a，执行操作后 `q` 停止；`@a` 回放，`@@` 再次回放 |

批量替换：`:%s/旧/新/gc` 在全文件逐项确认；选区后输入 `:s/旧/新/gc` 只处理选中行。只匹配完整单词可用 `:%s/\<旧词\>/新词/gc`。

查映射来源可用 `:verbose nmap f`、`:verbose omap f`、`:verbose nmap ,`、`:verbose xmap S`。不要同时启用另一个接管 `f/F/t/T/;/,` 的插件；插件加载顺序会决定最终生效者。

## 安装、维护与验证

将本目录作为 Neovim 配置目录使用；实际位置可通过 `:echo stdpath('config')` 查询。首次启动会引导 lazy.nvim，按 `lazy-lock.json` 恢复版本使用 `:Lazy restore`，外部语言工具通过 `:Mason` 管理。

- `Space ps` 打开插件列表；`Space pu` 检查更新，`Space pU` 执行插件更新。
- 修改社区导入、Leader 或查找映射后，重启 Neovim，避免旧进程残留映射和重复状态。
- 修改诊断规则只需 `:LspIgnoreReload`。
- 按需执行 `:checkhealth`、`:checkhealth vim.lsp`、`:checkhealth vim.provider` 排查环境问题。
- 配置回归从本目录运行以下命令；测试启动独立 Neovim 进程和临时缓冲区，不编辑业务文件。

```sh
# 完整 AstroNvim 配置：中文查找、原生语义对照、surround 组合
nvim --headless -u NONE -i NONE -l tests/chinese_find.lua

# 只有原生 Neovim + 已安装的 chinese_find.nvim
nvim --headless -u NONE -i NONE -l tests/chinese_find.lua standalone

# 单文件运行终端：真实按键切换、交互输入与退出后的输出保留
nvim --headless -u NONE -i NONE -l tests/file_runner.lua
```

独立插件仓库也自带无第三方依赖的回归测试：在其目录下运行 `nvim --headless -u NONE -i NONE -l tests/chinese_find.lua`。源码、许可证和独立测试维护于 [pu-007/chinese_find.nvim](https://github.com/pu-007/chinese_find.nvim)；本配置保留 AstroNvim 及 surround/jieba 的集成验证。

本次修复移除了 Leap 字符查找接管及手工 repeat_tick 同步。独立模块只记录目标字符及方向，并让表达式映射记录可重放的命令；最终移动由原生 f/F/t/T 完成。`,` 与 LocalLeader 的前缀竞争已解除，`j/k` 的 AstroCore 覆盖也显式关闭了继承的 `expr` 属性。

回归覆盖中英混合标点、全角 `df；/dt；`、顿号、计数、连续点重复、宏、可视模式、失败查找、t/T 邻接边界及强制 motion。Surround 通过 AstroCommunity 导入，不另外实现一套包围符引擎。

本次实际验证结果（独立 Neovim 进程、附加 UI 后输入真实按键；对比光标、缓冲区内容、模式和寄存器）：

| 验证范围                          | 结果                                                                          |
| --------------------------------- | ----------------------------------------------------------------------------- |
| 完整配置的字符查找与原生 Vim 对照 | 44 个场景、129 次断言通过                                                     |
| 独立插件仓库的原生行为对照        | 44 个场景、129 次断言通过                                                     |
| Surround / jieba 集成             | 10 个场景通过，含加删换包围符、中文选词、`yst；)`、surround 与 `ciw` 的点重复 |

原有待办中的「gpt astra 完善检查」仍保留，尚未限定具体功能范围。

参考：[AstroNvim 配置文档](https://docs.astronvim.com/configuration/)、[AstroCommunity](https://github.com/AstroNvim/astrocommunity)、[nvim-surround 操作说明](https://github.com/kylechui/nvim-surround/blob/main/doc/nvim-surround.txt)。
