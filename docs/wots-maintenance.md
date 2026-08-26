# WOTS 维护方式 / Maintenance Guide

> 日常维护、发布与故障排查手册。

---

## 日常维护

### 修改 Rust 代码后的标准流程

```bash
just lint          # clippy -D warnings，必须零告警
just test          # 全部单元 + 集成测试通过
just build         # Release 构建 → ./wots + 更新 zsh 补全
./wots sync-dry    # 或 just sync-dry，干运行核对路径映射
```

`just build` 会自动把补全脚本写入 `zsh.user/.config/zsh/completions/_wots`，
该文件随仓库版本化，所以构建后通常会产生一个待提交的补全 diff——正常现象，
随功能变更一起提交。

### 系统级周期维护

```bash
just refresh       # 保护工作区 → 更新依赖 → 备份元数据 → 清理缓存 → pull/push → 恢复
```

建议频率：每周一次或重大变更前。详见 [wots-pitfalls.md](wots-pitfalls.md) 第 18–20 条。

---

## 新增一个包类型（如未来再加 `.winxxx`）

1. `types.rs`：加变体 + `value()`/`FromStr`/`suffix()`/`sync_target()`/策略方法，
   并更新 `ALL_TYPES` / `SYNCABLE_TYPES` 常量。
2. `config.rs`：新增对应 `*_TARGET` LazyLock 路径。
3. `discover.rs`：`build_win_path` 加分支；如需自动检测则改 `detect_type`。
4. `create.rs`：`strip_base_for` 加分支。
5. 编译器会指出所有遗漏的 match 分支——这是本设计的核心安全网。
6. 补测试：类型转换、`build_win_path`、`compute_dest`、`find_packages`。

## 移除一个包类型

1. 删除变体并让编译器暴露全部引用点。
2. 在 `types.rs::from_str` 测试中保留"已移除类型必须解析失败"的断言，
   防止将来误恢复。
3. 检查仓库中是否存在使用旧后缀的目录，迁移数据后提交。

## 发布二进制

- `./wots` 是提交到仓库的预编译产物（~1.7 MB）。改动代码后必须 `just build`
  保持二进制与源码同步提交。
- 若未来想停止版本化二进制：加入 `.gitignore`、删除根目录 `wots`，
  README 的"声明"部分已说明用户可自行 `just build`。

---

## 故障排查

| 症状 | 排查 |
| ---- | ---- |
| sync 显示 "missing --win-user" | 传 `--win-user` 或设 `WIN_USER` |
| stow 报 not owned by stow | 正常降级为逐文件 ln；如需 stow 管理，手动删除目标侧旧文件后重试 |
| Windows 文件没更新但显示 copied | 删除 `.wots_index.json` 后重新 `diff`/`sync` |
| robocopy 失败 exit code ≥ 8 | 手动跑一条 robocopy 看 UNC 路径是否可达：`ls '\\wsl$\<distro>'`（pwsh） |
| diff 大量 missing-win | 可能是 WIN_USER 改名了——目标路径随之改变，属预期 |
| create 后目标路径不对 | 用 `-n` 干运行检查 `compute_dest` 映射；确认源在正确基路径下 |

## 测试约定

- 单元测试与实现同文件的 `#[cfg(test)] mod tests`。
- 集成测试统一放 `tests/integration.rs`，临时目录按 pid+时间戳隔离；
  同一模块内共享 temp 目录的测试要注意相互污染（历史上踩过）。
- 依赖 WSL 挂载的场景以 `/mnt/c/Windows` 存在性守卫并在无挂载时跳过。
- 回归修复时先写复现测试再修码（索引毒化回归即范例）。

## 已知技术债

- pwsh 目录拷贝回退对已存在目标目录的行为不够严格（见 pitfalls 第 8 条）。
- `sync_batch` 的计数用 `HashMap<String, usize>` 字符串键，可升级为强类型结构体。
- 集成测试中部分场景断言较宽松（时间戳倾斜容忍），可引入时钟注入收紧。
