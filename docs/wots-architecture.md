# WOTS 技术实现 / Technical Implementation

> 面向贡献者与审阅者的实现说明。使用方法见 [wots-usage.md](wots-usage.md)。

---

## 总体架构

```
┌─────────────┐     ┌──────────────────────────────────────────┐
│ justfile    │ →   │ wots (Rust binary, ~1.7 MB, 零运行时依赖) │
│ 任务编排    │     │  main.rs ── 解析 CLI 并分发               │
└─────────────┘     │  cli.rs ──── clap derive 定义             │
                    │  types.rs ── PkgType 枚举 + 路径映射规则   │
                    │  config.rs ─ LazyLock 全局配置            │
                    │  discover.rs 包发现 / 类型检测 / 路径构建  │
                    │  status.rs ─ 同步状态检查（哈希/索引）      │
                    │  sync.rs ─── stow + robocopy/pwsh 编排     │
                    │  create.rs ─ 包创建（原子拷贝）            │
                    │  commands.rs stats/list/diff 实现          │
                    │  display.rs ─ 表格渲染 + 交互提示          │
                    │  index.rs ─── SyncIndex 数据模型           │
                    │  util.rs ──── 文件系统工具                 │
                    └──────────────────────────────────────────┘
```

单一二进制、无插件系统；所有行为由 `PkgType` 的方法表驱动（`sync_target` /
`uses_stow` / `uses_copy_sync` / `needs_sudo`），新增类型只需改 `types.rs` +
`config.rs` 并补齐 match 分支，编译器会强制覆盖所有调用点。

---

## 关键机制

### 1. 类型检测（discover.rs）

- **仓库内**：纯后缀匹配（`type_from_dir_name`），不读目录内容。
- **create 自动检测**（`detect_type`）：基于路径前缀推断——
  `/mnt/c/Users/<name>/…` → WinUser；`/mnt/c/其他` → WinRoot；
  `$HOME/.config/…` → Config；`$HOME` 下其余 → User；
  `/proc|/sys|/dev|/run|/tmp` → Meta；其余绝对路径 → Root。

### 2. Windows 目标路径构建（build_win_path）

```
repo 文件 ──(剥掉包根或 <win-user> 层)──► rel
WinUser:  MNT_C/Users/{user}/rel
WinRoot:  MNT_C/rel
```

### 3. Linux 符号链接同步（sync.rs）

1. 优先 `stow --adopt -t <target> <pkg>`（cwd = DOTFILES_DIR）。
2. 冲突（"not owned by stow"）→ 逐文件 `ln -sf`。
3. root 类型直接走 `sudo ln -sf` + `sudo mkdir -p`。

### 4. Windows 拷贝同步（sync.rs）

1. 检测 WSL（`/proc/sys/fs/binfmt_misc/WSLInterop`）、robocopy/pwsh 可用性。
2. 目录：`robocopy.exe \\wsl$\{distro}\… C:\… /MIR /MT:8 /R:1 /W:1`。
3. 单文件：`robocopy <src_dir> <dst_dir> <file>`（目标是**父目录**，避免生成同名目录嵌套）。
4. 回退：`pwsh.exe -NoProfile -Command Copy-Item …`（CMD 不支持 UNC，故必须 pwsh）。
5. robocopy 退出码 `< 8` 视为成功（0–7 是警告级别）。

### 5. 同步状态索引（index.rs + status.rs）

`.wots_index.json` 记录每个文件的 `mtime_ns / size / win_mtime_ns / win_size /
blake3_wsl / blake3_win / synced`：

- **快速路径**：索引标记已同步且双方 mtime+size 均未变 → 直接判 Synced，不做哈希。
- **慢速路径**：元数据一致时计算 blake3 双侧哈希比对；不一致按时间戳判定
  NeedsSync（本地新）/ NewerOnWin（远端新）/ ContentChanged（内容分叉但 mtime 相同）。
- **反向检查**（detect_missing_wsl）：扫描索引中 WSL 侧已消失的键，若目标侧仍存在则报 MissingWsl，双侧均无则清理索引项。
- 写入为 `.tmp` → `rename` 原子替换；JSON 解析失败降级为空索引并告警。

### 6. 包创建（create.rs）

- move 操作 = copy 到 `.wots_tmp_<pid>` 后缀 → 数量/大小校验 → 原子 rename → 删除源。
- `compute_dest` 按 `strip_base_for(type)` 剥掉对应基路径（HOME / ~/.config / /
  / MNT_C 映射），保持包内相对布局与目标一致。
- Windows 类型强制要求 `--win-user`/`WIN_USER` 已设置，防止落到默认 "user"。

---

## 测试策略（tests/integration.rs + 各模块 #[cfg(test)]）

- **单元测试**：所有模块均有，覆盖纯函数（路径映射、类型转换、格式化）。
- **集成测试**：在临时目录构造真实文件，跑完整状态机：
  双侧同步 / WSL 编辑 / Windows 编辑 / 单侧删除 / 双侧删除（含索引清理回归）/
  索引毒化回归（编辑后的状态必须在第二次检查中保持）。依赖 `/mnt/c/Windows`
  存在的场景在没有 WSL 挂载的环境自动跳过。
- 运行：`just test`；静态检查：`just lint`（clippy `-D warnings`）。

---

## 构建

```bash
cargo build --release --manifest-path wots-src/Cargo.toml
cp wots-src/target/release/wots ./wots
# 或
just build
```

Release profile：`opt-level = 3`、`lto = true`、`codegen-units = 1`。

依赖全部为纯 Rust crate：`clap`(+complete)、`walkdir`、`rayon`、`serde`(json)、
`anyhow`、`colored`、`glob`、`shellexpand`、`blake3`、`comfy-table`。

运行时外部工具：GNU Stow（Linux）、robocopy.exe 或 pwsh.exe（Windows 同步）。
