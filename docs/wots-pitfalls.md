# WOTS 可能踩的坑 / Pitfalls & Gotchas

> 实际使用中容易出错或需要留意的点。

---

## 路径与类型

1. **后缀即契约**。目录名后缀决定一切行为；改名（如 `foo.user` → `foo.config`）
   会同时改变目标路径和同步引擎。重命名包后必须重新 `sync`。
2. **`.winuser` 包内的用户名层是可选的**。若包内存在与 Windows 用户名同名的
   子目录，该层会被剥掉再映射；否则整个包根视为相对布局。两种结构混用同一包
   时要小心重复映射。
3. **Windows 类型源文件必须在 `/mnt/c` 下**。`create` 会拒绝其他位置的源，
   防止把 WSL 侧文件错误地当作 Windows 侧现状拷贝。
4. **`detect_type` 的自动检测只是猜测**。跨类型批量 create 前先 `--dry-run`
   看清目标路径，必要时显式 `-t`。
5. **`/mnt/c` 挂载点可用 `WSL_MNT` 环境变量覆盖**，但 Windows 侧路径硬编码为
   `C:\` 语义——非默认盘符环境不要改这个变量来做双盘同步。

## 同步方向

6. **robocopy `/MIR` 是镜像 = 会删除**。目标侧多出来的文件会被删掉。首次同步
   一个新包前先 `just sync-dry` 或依赖默认的路径确认清单核对。
7. **单文件 robocopy 的目标是父目录**。实现里已处理（取 dst 父目录 + 文件名），
   但如果你绕过 wots 手写 robocopy 命令，把文件路径当目标目录会得到同名目录嵌套。
8. **pwsh 回退路径的目录拷贝**：当目标目录已存在时 PowerShell `Copy-Item
   -Recurse` 有"拷贝成子目录"的历史怪癖；wots 只在 robocopy 完全不可用时才走
   这条路。优先保证 `robocopy.exe` 在 PATH 中。
9. **双向编辑会分叉**：两侧 mtime 不同时按"谁新谁赢"判定，但 sync 只做
   WSL → Windows 单向推送。Windows 侧修改过的文件（newer-on-win）不会自动拉回，
   需要手动合并进仓库。
10. **mtime 精度**：跨文件系统（ext4 ↔ NTFS via 9p）时间戳有舍入，状态检查有
    2ms 容差。刚拷贝完立即 diff 出现 synced 属正常现象。

## 权限与执行环境

11. **root 包用 `sudo ln -sf`**，stow 不参与；sudo 会话过期时 `just sync-root`
    会逐条失败，先 `sudo -v` 刷新凭据。
12. **WinRoot 写 `/mnt/c` 不需要 sudo**（默认 WSL 挂载），但如果修改了挂载选项
    （如 `metadata` 未启用、uid 映射变化），写入可能被拒。
13. **不在 WSL 环境（原生 Linux）下 Windows 同步整体跳过并告警**，不是错误；
    CI 中可安全运行其余命令。
14. **`pwsh.exe` / `robocopy.exe` 探测靠实际调用**，首次运行可能因启动 Windows
    interop 略慢。

## 索引与状态

15. **`.wots_index.json` 是缓存不是真相**。怀疑状态不对时删掉它即可重建
    （代价是全量哈希比较）。它已加入 git-ignore 与排除模式。
16. **索引按 `包名/相对路径` 作键**。两个不同类型的包如果 basename 相同
    （如 `git.user` 与 `git.winuser`），键空间互不干扰（含完整目录名），
    但 `--app git` 无后缀匹配会同时命中两者——确认这是你想要的。
17. **超过 `WOTS_MAX_SIZE_MB`（默认 50MB）的文件被标记 skipped**，既不比较也
    不同步，注意大配置数据库类文件不会被保护。

## Just 工作流

18. **`just refresh` 会 stash 未提交修改并在结束后 pop**。若 pop 冲突，stash
    仍保留在栈中，按提示手动恢复，不要反复跑 refresh。
19. **CONFIRM 变量只影响 just 包装器**。直接调用 `wots sync` 时默认同样要求
    确认，传 `-y` 跳过。
20. **`backup` 步骤会自动 commit 元数据变更**（--no-verify）。不希望自动提交
    就单独跑各子命令而不要用 `refresh`。
