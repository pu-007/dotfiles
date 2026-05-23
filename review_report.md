# WOTS Rust Rewrite — Code Review Report

**Date**: 2026-05-23  
**Reviewed**: `wots/src/` (10 modules, ~1500 lines Rust)  
**Scope**: Logic errors, performance bottlenecks, correctness, edge cases (root directory, create, stat/list/diff)

---

## 1. Critical Bugs (Correctness)

### 1.1 `create`: User Type Override Silently Ignored

**File**: `create.rs`, lines 64-73  
**Severity**: High — produces wrong package type

When the user interactively overrides the detected package type (e.g., detected `user` but user types `config`), the new type is matched but **never assigned to `pt`**. The original detected type is silently returned, so the package is created with the wrong type suffix and target.

```rust
// Line 64-73: new_pt is recognized but not assigned to `pt`
if resp != "y" && !resp.is_empty() && resp != pt.value() {
    match PkgType::from_str(&resp) {
        Some(new_pt) => {
            display::info(&format!("Using type: {}", new_pt.value()));
            // BUG: pt is never reassigned to new_pt
        }
        ...
    }
}
// Line 77: returns the ORIGINAL detected pt, not the override
pt
```

**Fix direction**: Add `pt = new_pt;` after the `Some(new_pt)` match arm, or restructure so the override path returns the correct value.

---

### 1.2 `status`: Windows-Newer Files Not Tracked

**File**: `status.rs`, lines 226-235  
**Severity**: Medium — missing state in status reporting

In `check_copy_status`, the comparison only increments `outdated_local` when `wsm > wnm` (local mtime newer than Windows). When the Windows file is **newer** (`wsm < wnm`), **no counter is incremented**. This means files modified on Windows and not yet synced back are silently dropped from the status tally.

```rust
if mtime_diff < 1 && ws.len() == wn.len() {
    counts.synced += 1;
} else if wsm > wnm {
    counts.outdated_local += 1;
}
// Missing: wsm < wnm case — Windows newer than local
```

**Fix direction**: Add a third branch for the `wsm < wnm` case (e.g., `counts.outdated_remote` or similar), or fold it into a general "diverged" count.

---

### 1.3 `diff`: `--json` Flag Accepted But Never Used

**File**: `main.rs`, lines 229-322; `cli.rs`, line 99  
**Severity**: Low — user-facing dead parameter

`DiffArgs` declares `--json` / `-j` flag, but `cmd_diff` never checks `args.json_output`. The flag is accepted silently and ignored, producing text output regardless.

**Fix direction**: Either implement JSON output for `cmd_diff` or remove the `json_output` field from `DiffArgs`.

---

## 2. Logic Errors

### 2.1 `propose_name`: Single Files Always Use Parent Directory

**File**: `discover.rs`, lines 167-195  
**Severity**: Medium — wrong default app name

The condition on line 181:
```rust
if (init_names.contains(&file_name.as_str()) || sources.len() == 1)
```

is logically wrong. `sources.len() == 1` makes ALL single-file creates use the parent directory name, not just init-like files. For example, `wots create ~/.zshrc` would propose the **user's home directory name** (e.g., `pu`) instead of `zshrc`.

The plan (section 4.2) explicitly intended this fallback only for init-like names (`init.lua`, `init.vim`, `config`, etc.), not for all single sources.

**Fix direction**: Remove `sources.len() == 1` from the condition. The parent-name fallback should only trigger for files named `init.lua`, `init.vim`, `config`, `config.yaml`, `settings.json`.

---

### 2.2 `detect_type`: Root Detection Too Narrow

**File**: `discover.rs`, lines 58-59  
**Severity**: Medium — root packages not detected for non-`/etc` paths

```rust
if rp.starts_with(&*ROOT_TARGET.join("etc")) || rp == *ROOT_TARGET {
    return PkgType::Root;
}
```

This only detects root type for files under `/etc` (and `/` itself). Files under `/usr/share/`, `/opt/`, `/var/`, or any other root-level directory are classified as `Meta` instead of `Root`. The plan intended `Root` to cover all files under `/` that are not in HOME.

**Fix direction**: Change the condition to cover all paths under `/` that are not under HOME, or use a more general check like `rp.starts_with(ROOT_TARGET) && !rp.starts_with(HOME)`.

---

### 2.3 `validate_sources`: Rejects `/tmp` and Other Non-HOME Paths

**File**: `create.rs`, lines 155-190  
**Severity**: Medium — contradicts plan's goal

The `validate_sources` function for Linux config types requires `src.starts_with(HOME)`. Any source outside HOME (e.g., `/tmp/myconfig`) is rejected. The analysis report (section 2.1, item 1) and the plan (section 4.2) both flagged this as a problem to fix, but the Rust implementation retains the same restriction.

**Fix direction**: Allow sources from any path for user/config/local types, or at least add a `--force`/`--allow-external` flag. The path-mapping in `compute_dest` (stripping HOME) can fall back to using only the filename when HOME isn't a prefix.

---

### 2.4 `compute_dest`: Fragile String Replacement for Windows Paths

**File**: `create.rs`, lines 200-201  
**Severity**: Low — fragile path handling

```rust
let target_mnt = PathBuf::from(
    target.to_string_lossy().replace("C:", &MNT_C.to_string_lossy())
);
```

This string-based `C:` → `/mnt/c` replacement is fragile. If `MNT_C` is not exactly `/mnt/c` (it's configurable via `WSL_MNT`), or if `C:` appears in a directory name, the replacement produces a wrong path. Also, `C:` as uppercase is assumed but Windows paths from `PathBuf` may use lowercase.

**Fix direction**: Use proper path component manipulation (strip the drive letter and join with MNT_C) rather than string replacement.

---

## 3. Performance Bottlenecks

### 3.1 Sync Index Is Dead Code

**File**: `status.rs`, lines 11-59  
**Severity**: High — key planned optimization is unimplemented

The `SyncIndex` struct with `load()`, `save()`, `get()`, `set()` is fully implemented but **never called** by any other module. The plan (section 4.1) explicitly designed this as the primary performance optimization — skipping `/mnt/c` `stat` calls when mtime+size haven't changed. Without it, `stats` and `list` still perform full scans across the WSL/Windows boundary.

**Fix direction**: Integrate `SyncIndex` into `check_copy_status` and `check_stow_status`:
1. Load index before scanning
2. Compare each file's current mtime+size against the index entry
3. Skip `/mnt/c` stat if the local file hasn't changed
4. Update index after scan

---

### 3.2 `sync_batch` Ignores Concurrency Limit

**File**: `sync.rs`, lines 234-296  
**Severity**: Medium — risk of Windows IO saturation

The `_max_concurrent` parameter is accepted but never used. `par_iter().with_max_len(1)` does NOT limit concurrency — it only controls internal chunking within rayon. The rayon thread pool processes as many items in parallel as there are CPU cores. Multiple concurrent `robocopy.exe` processes can saturate Windows-side IO, exactly the risk the plan warned against (section 4.6, section 8).

**Fix direction**: Either:
- Use a `Semaphore` or `rayon::ThreadPool` with limited threads
- Ensure `sync_batch` is called one package at a time (since `robocopy` has its own `/MT` concurrency)
- Or abandon per-file parallelism entirely and use one `robocopy` call per package directory

---

### 3.3 Double Directory Scan in `cmd_stats`

**File**: `main.rs`, lines 68-78  
**Severity**: Medium — redundant work

`cmd_stats` performs two separate parallel scans over each package directory:
1. `pkgs.par_iter().map(|p| util::count_files(p))` — counts files
2. `pkgs.par_iter().map(|p| util::dir_size(p))` — sums sizes

Both functions call `WalkDir`, so each package tree is traversed twice. These can be merged into a single scan that counts and measures simultaneously.

**Fix direction**: Create a combined `count_and_size(dir: &Path) -> (usize, u64)` function that does one WalkDir pass.

---

### 3.4 Sequential Status Checking

**File**: `status.rs`, lines 161-170, 241-252  
**Severity**: Low — missed parallelism opportunity

`check_stow_status_batch` and `check_copy_status_batch` iterate packages sequentially. For Windows packages under `/mnt/c`, each `stat` call has millisecond latency. These loops could use `par_iter()` from rayon for parallel checking.

**Fix direction**: Use `rayon::par_iter()` in batch functions, similar to how `cmd_stats` parallelizes file counting.

---

## 4. Root Directory Edge Cases

### 4.1 Root Symlink Potentially Follows Package Symlinks

**File**: `sync.rs`, line 195  
**Severity**: Low — information disclosure risk

```rust
let src = f.canonicalize().unwrap_or_else(|_| f.to_path_buf());
```

In `stow_file_by_file`, the source file path is canonicalized (resolving all symlinks) before creating the symlink. If a file within a root package is itself a symlink pointing outside the repository (e.g., pointing to a system file), the created symlink will point to that external location. This is normal stow behavior, but for root packages, the resolved path could expose unintended filesystem paths.

**Fix direction**: This is arguably correct behavior (matching GNU Stow). Document the behavior and consider a `--no-dereference` flag.

---

### 4.2 `is_symlink` Fallback Runs `sudo` Without Warning

**File**: `status.rs`, lines 84-93  
**Severity**: Low — silent sudo call

When `fs::symlink_metadata` fails with `PermissionDenied` (common for root-owned files), the code silently spawns `sudo test -L`. This could:
- Hang waiting for a sudo password prompt
- Produce unexpected security audit log entries
- Fail silently if passwordless sudo isn't configured

**Fix direction**: At minimum, log a `display::dim` message when falling back to sudo. Better: check if `pt.needs_sudo()` is true and handle the permission model explicitly rather than on-demand.

---

## 5. Additional Edge Cases

### 5.1 `create_atomic` Temp File Collision

**File**: `create.rs`, lines 228-231  
**Severity**: Low — unlikely but possible

```rust
let tmp = dest.with_extension(format!(".wots_tmp_{}", std::process::id()));
```

If the same PID is reused (possible if the previous process exited) and the temp file wasn't cleaned up from a prior failed run, the `rename` could fail or overwrite. The PID alone is not a unique-enough suffix for a race-free temp file.

**Fix direction**: Use a random suffix (`std::time::SystemTime::now().duration_since(...)` nanoseconds, or a UUID) instead of PID.

---

### 5.2 `is_excluded` Checks Every Path Component Against Every Pattern

**File**: `util.rs`, lines 102-113  
**Severity**: Low — performance edge

For a deep path like `/home/pu/dotfiles/packages.user/app/.config/sub/deep/file.txt`, every component is matched against all 10 exclude patterns. The `.config` component would be compared against `.git`, `__pycache__`, etc. — all false, but O(depth × patterns) per file. This is minor but could be optimized.

**Fix direction**: Match only the file name or the last few components, not every component.

---

### 5.3 `pwsh_copy` Uses `xcopy` (Deprecated)

**File**: `sync.rs`, line 393  
**Severity**: Low — tool deprecation

The pwsh fallback uses `xcopy` which has been deprecated in modern Windows in favor of `robocopy`. The plan explicitly stated using `robocopy` as the primary engine, but the fallback still uses `xcopy`/`copy`. This is acceptable as a fallback but worth noting.

---

### 5.4 `glob_match` Not Used for `*.pyc` Type Patterns

**File**: `util.rs`, lines 119-128  
**Severity**: Low — wildcard patterns work but validation could improve

The `glob_match` function correctly handles `*` and `?` patterns via the `glob` crate. However, `EXCLUDE_PATTERNS` includes `*.pyc` — the `glob` crate matches `*.pyc` against whole strings, not substrings. Since `is_excluded` checks individual path components, this works correctly. But the initial `pattern == name` short-circuit means `*.pyc` would not equal a `.pyc` file name, so it correctly falls through to the glob match. OK.

---

## 6. Unused Code

| Location | Symbol | Notes |
|---|---|---|
| `status.rs:11-59` | `SyncIndex`, `IndexEntry` | Full index system — never called |
| `status.rs:254-260` | `index_key` | Helper function — never called |
| `util.rs:130-132` | `skip_size_limit` | Never called |
| `main.rs:99` (cli) | `DiffArgs.json_output` | Parsed but never used |
| `config.rs:70-84` | `SYNC_MAX_CONCURRENT` | Read but `sync_batch` ignores it |

---

## 7. Summary

| Category | Count | Severity |
|---|---|---|
| Critical Bugs | 1 | High |
| Logic Errors | 4 | Medium–Low |
| Performance Issues | 4 | High–Low |
| Unused Code | 5 items | — |
| Edge Cases | 4 | Low |

### Top Priority Fixes

1. **`create`: user type override not applied** — silently creates wrong package type
2. **Sync index not integrated** — major performance loss on `/mnt/c` operations
3. **`propose_name`: wrong default name for single files** — e.g., `~/.zshrc` → `pu.user` instead of `zshrc.user`
4. **Concurrency ignored in `sync_batch`** — risk of Windows IO saturation
5. **Double scan in `cmd_stats`** — merge file count + size into one pass
6. **Root detection too narrow** — only catches `/etc/*`, not `/usr/*`, `/opt/*`, etc.
