# AstroNvim Template

**NOTE:** This is for AstroNvim v6+

A template for getting started with [AstroNvim](https://github.com/AstroNvim/AstroNvim)

## 🛠️ Installation

#### Make a backup of your current nvim and shared folder

```shell
mv ~/.config/nvim ~/.config/nvim.bak
mv ~/.local/share/nvim ~/.local/share/nvim.bak
mv ~/.local/state/nvim ~/.local/state/nvim.bak
mv ~/.cache/nvim ~/.cache/nvim.bak
```

#### Create a new user repository from this template

Press the "Use this template" button above to create a new repository to store your user configuration.

You can also just clone this repository directly if you do not want to track your user configuration in GitHub.

#### Clone the repository

```shell
git clone https://github.com/<your_user>/<your_repository> ~/.config/nvim
```

#### Start Neovim

```shell
nvim
```

## 本地功能

- `lua/diagnostic_ignores.lua`：批量忽略 LSP 警告的规则文件。`:LspIgnoreEdit` 打开、`:LspIgnoreReload` 重载（取消规则后诊断会恢复）。默认忽略 marksman 的 `Ambiguous link to document '...'` 类警告；按服务器、严重级别、完整消息 / 诊断码 / Lua 模式批量配置，示例见文件内注释。
- `lua/plugins/chinese_find.lua`：基于 [leap.nvim](https://codeberg.org/andyg/leap.nvim) 的中英标点等价 `f/F/t/T`——`f,` 同时匹配 `,` 与 `，`（`.。` `;；` `?？` 引号等同理），仅限当前行；`2f,`、`;`/`,` 重复、`df,` 后 `.` 重复均可用。等价组在该文件顶部修改。注意 `,` 同时是 `<LocalLeader>`：当前无本地快捷键冲突，若日后添加 `<LocalLeader>x` 映射，裸按 `,` 会等待 `timeoutlen`。
- `lua/plugins/code_runner.lua` + `lua/file_runner.lua`：单文件快速运行。`<Leader>rr` 保存并运行当前文件（复用同一个 snacks 终端，可交互输入），`<Leader>rt` 显示/隐藏运行终端；`:RunFile` 等为 code_runner 原生命令（每次新开终端）。支持 python（优先 venv-selector 选中的解释器）、c/cpp、rust 单文件、javascript、sh、lua；TypeScript 的 tsx/Deno 示例见 `file_runner.lua` 注释。
- 插件配置按功能拆分：`lua/plugins/appearance.lua`（外观）、`editing.lua`（编辑）、`chinese_input.lua`（中文输入）、`chinese_find.lua`（中文查找）、`code_runner.lua`（运行）。
