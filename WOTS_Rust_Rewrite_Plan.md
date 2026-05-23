# WOTS (WSL Dotfile Tool) Rust 重构技术蓝图

## 1. 现状深层诊断：为什么 Python 方案难以为继？

### 2.1 `create` 命令的灵活性与识别缺陷
目前的 `create` 逻辑（`cli.py` & `discover.py`）存在“路径决定论”的局限：
*   **硬编码推断**：`detect_type` 严重依赖绝对路径匹配。如果你在临时文件夹或 Git Repo 内部运行 `create`，它会因无法匹配 `$HOME` 或 `/mnt/c/Users` 而识别失败。
*   **App 命名冲突**：默认使用首个文件的 `stem` 作为包名。对于像 `.config/nvim/init.lua` 这样的文件，它可能识别为 `init.config` 而非 `nvim.config`。
*   **多源文件混乱**：同时添加 `~/.zshrc` 和 `~/.config/nvim/` 时，`_compute_dest` 的相对路径基准（`Path.home()`）会导致它们在目标包中层级参差不齐，缺乏统一的“根路径”概念。

### 2.2 搜索与列表（Discovery）的逻辑漏洞
*   **全量 IO 负担**：`list` 和 `stats` 命令为了显示状态，对每个包的所有文件都执行了 `stat` 操作。在 WSL 的 `/mnt/c` 挂载点上，`stat` 的延迟（Metadata Latency）是毫秒级的。1000 个文件意味着数秒的等待。
*   **缺少并行深度控制**：Python 方案虽然用了 `asyncio`，但本质上是 `to_thread` 包装。在高并发下，WSL 对 Windows 文件系统的处理会由于 IO 队列饱和而发生拥塞。

### 2.3 部署依赖困境
*   **引导（Bootstrapping）问题**：在新机器上执行 `git clone` 后，你面临“先有鸡还是先有蛋”的问题。即使有 `pixi`，也需要先安装 `pixi` 环境才能运行 `wots`。Rust 编译出的单二进制文件（Standalone Binary）可以放入 Git 或作为 Release 下载，真正实现零依赖运行。

---

## 2. Rust 重构目标：高性能、类型安全、零依赖

### 2.1 核心改进目标
1.  **极速扫描**：利用 Rust 的多线程和 `walkdir` / `ignore` 库，结合 `jwalk` 实现并行元数据查询。
2.  **增量同步（The "Sync-State" Index）**：引入轻量级状态数据库（如 SQLite 或 JSON 索引），仅对比文件大小和修改时间。
3.  **智能包创建**：允许 `--base` 显式指定源路径基准，增强 App 命名逻辑（如自动截取父级目录名）。
4.  **Robocopy 集成**：Windows 侧同步不再通过 `pwsh + copy`，而是直接调用底层的 `robocopy.exe`（支持多线程、增量同步和权限保留）。

---

## 3. Rust 项目架构设计

我们将项目命名为 `wots-rs` (或直接替换为 `wots`)。

### 3.1 模块结构 (Directory Structure)
```text
wots/
├── Cargo.toml
├── src/
│   ├── main.rs          # 入口点，CLI 路由
│   ├── cli/             # 命令行解析 (clap)
│   │   ├── create.rs    # Create 命令实现
│   │   ├── sync.rs      # Sync 命令实现
│   │   └── list.rs      # List/Stats 命令实现
│   ├── core/            # 核心领域模型
│   │   ├── package.rs   # PkgType, Package 定义
│   │   ├── discovery.rs # 扫描与类型推断逻辑
│   │   └── config.rs    # 全局配置处理 (Path, Env)
│   ├── engine/          # 执行引擎
│   │   ├── stow.rs      # Linux Stow (Symlink) 逻辑
│   │   └── windows.rs   # Windows Robocopy 同步逻辑
│   └── utils/           # 通用工具 (FS, Path 翻译)
└── tests/               # 集成测试
```

### 3.2 关键技术栈
*   **CLI**: `clap` (v4, derive 模式) - 工业级 CLI 解析。
*   **FS Operations**: `walkdir` + `fs_extra` - 高性能文件遍历。
*   **Async/Concurrency**: `tokio` (主要用于管理并发进程) 或 `rayon` (用于 CPU 密集型文件树遍历)。
*   **Serialization**: `serde` + `toml` - 处理配置和状态索引。
*   **Path Translation**: `typed-path` - 专门处理 Unix 和 Windows 路径的跨平台转换。
*   **Error Handling**: `anyhow` + `thiserror` - 健壮的错误链路追踪。

---

## 4. 关键逻辑重构方案

### 4.1 智能路径识别 (Intelligent Identification)
在 `create` 时，不再仅依赖 `detect_type`：
```rust
// 伪代码示例
pub fn identify_package(sources: &[PathBuf]) -> Result<PackageProposal> {
    // 1. 找到所有源文件的公共最小父目录 (LCP)
    // 2. 检查 LCP 是否在 HOME, /etc 或 /mnt/c 下
    // 3. 如果识别模糊，允许用户通过 CLI 交互从多个候选列表中选择
}
```

### 4.2 Windows 同步引擎优化
不再使用 `copy` 循环：
```rust
// 核心优化：利用 Robocopy 的原生并发和差异检查
pub fn sync_to_windows(src: &Path, dst: &Path) {
    let output = Command::new("robocopy.exe")
        .arg(src)
        .arg(dst)
        .arg("/MIR")  // 镜像模式
        .arg("/MT:8") // 8线程并发
        .arg("/XF").arg(".git") // 排除
        .spawn();
}
```

---

## 5. 迁移路线图 (Roadmap)

1.  **Phase 1: CLI & Discovery (1周)**
    *   建立 `clap` 命令行框架。
    *   实现基于后缀的包扫描逻辑。
2.  **Phase 2: Sync Engines (2周)**
    *   实现 Linux 侧的符号链接管理（兼容 `GNU Stow` 行为）。
    *   实现基于 `robocopy` 的 Windows 侧高效同步。
3.  **Phase 3: Intelligence & Safety (1周)**
    *   重写 `create` 命令，支持原子操作和更灵活的路径推断。
    *   加入文件变化预览（`diff` 功能）。
4.  **Phase 4: Release & Cleanup (1周)**
    *   通过 GitHub Actions 交叉编译 WSL 二进制文件。
    *   更新 `justfile` 调用 Rust 编译出的 `wots` 替代 Python 模块。

---

## 6. 总结
重构为 Rust 不仅仅是速度的提升，更是**系统鲁棒性**的质变。Rust 的类型系统将确保在处理复杂的跨文件系统（WSL/Windows）操作时不会出现悬挂的符号链接或意外的文件覆盖。这与你 README 中提到的“Transform into a robust pipeline”目标完全契合。
