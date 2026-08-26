# 点文件仓库维护

> 引擎开发（改 Rust、增删类型、发布二进制）见 [`../../wots-src/docs/maintenance.md`](../../wots-src/docs/maintenance.md)。

---

## 日常

```bash
just refresh       # 保护工作区 → 更新依赖 → 备份元数据 → 清理缓存 → pull/push → 恢复
just sync-dry      # 核对路径后再 just sync
```

建议每周一次或重大变更前跑 `refresh`。注意：

- `refresh` 会 stash 未提交修改并在结束后 pop；pop 冲突时 stash 仍在栈中，不要反复跑。
- `backup` 会 `--no-verify` 自动 commit 元数据变更；不想自动提交就不要用 `refresh`。

`./wots` 由 `~/wots-src` 的 `just build` 安装。改引擎后到那边构建，再回本仓库提交二进制与补全 diff。

---

## 故障排查

| 症状 | 排查 |
| ---- | ---- |
| sync 显示 "missing --win-user" | 传 `--win-user` 或设 `WIN_USER` |
| stow 报 not owned by stow | 正常降级为逐文件 ln；要 stow 管理则删目标侧旧文件后重试 |
| Windows 文件没更新但显示 copied | 删除 `.wots_index.json` 后重新 `diff`/`sync` |
| robocopy 失败 exit code ≥ 8 | 在 pwsh 检查 `ls '\\wsl$\<distro>'` |
| diff 大量 missing-win | `WIN_USER` 可能改名了 |
| create 后目标路径不对 | `-n` 干运行核对；确认源在正确基路径下 |
