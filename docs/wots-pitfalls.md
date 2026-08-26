# 使用本仓库时容易踩的坑

> 引擎实现层面的坑见 [`../../wots-src/docs/pitfalls.md`](../../wots-src/docs/pitfalls.md)。

1. **后缀即契约**。`foo.user` 改成 `foo.config` 会同时改目标路径和同步方式，改完必须重新 `sync`。
2. **robocopy `/MIR` 会删目标侧多余文件**。新包先 `just sync-dry`。
3. **sync 只推 WSL → Windows**。Windows 侧改过的文件（newer-on-win）不会自动拉回，需手动并进仓库。
4. **root 包要 sudo**。凭据过期时先 `sudo -v` 再 `just sync-root`。
5. **`.wots_index.json` 是缓存**。状态不对就删掉重建（已 gitignore）。
6. **`just refresh` 会 stash / pop**。pop 冲突时 stash 仍在栈中，不要连跑。
7. **`CONFIRM` 只影响 just 包装器**。直接 `./wots sync` 默认也要确认，`-y` 跳过。
8. **`backup` 会自动 commit 元数据**（`--no-verify`）。不想自动提交就别用 `refresh`。
