"""
CLI — typer app with argparse fallback.

Commands:
  create   Create a new package from existing files (interactive)
  sync     Sync (stow / copy) packages to their targets
  stats    Show repository statistics
  list     List all packages with details
"""
from __future__ import annotations

import sys, json, shutil
from pathlib import Path
from typing import Optional, List, Dict, Tuple

from . import config
from .types import PkgType, type_label
from .discover import detect_type, find_packages, pkg_basename, build_mnt_path
from .sync import do_stow, sync_one_file, sync_batch, prepare_sync_items, build_winuser_wsl_path
from .status import check_stow_status, check_copy_status, status_text
from .utils import count_files, dir_size, fmt_size, is_wsl, has_pwsh, has_stow, is_excluded

HAS_TYPER = False
try:
    import typer
    from typer import Typer, Option, Argument, Exit
    HAS_TYPER = True
except ImportError:
    pass


# ═══════════════════════════════════════════════════════════════════════════════
#  Command: create
# ═══════════════════════════════════════════════════════════════════════════════

def cmd_create(
    sources: List[str],
    app_name: Optional[str] = None,
    pkg_type_str: Optional[str] = None,
    *,
    no_stow: bool = False,
    no_sync: bool = False,
    dry_run: bool = False,
    yes: bool = False,
):
    """Create a new package from existing files."""
    from .display import info, success, error, warning, dim, rule

    # 1 ─ resolve source paths
    resolved: List[Path] = []
    for s in sources:
        p = Path(s).expanduser().resolve()
        if not p.exists():
            error(f"Source does not exist: {s}")
            raise Exit(1)
        resolved.append(p)

    # 2 ─ determine type (with interactive confirmation)
    if pkg_type_str:
        try:
            pt = PkgType(pkg_type_str)
        except ValueError:
            valid = ", ".join(t.value for t in PkgType)
            error(f"Unknown type: {pkg_type_str}.  Use: {valid}")
            raise Exit(1)
    else:
        detected = {detect_type(p) for p in resolved}
        if len(detected) > 1:
            error("Sources have mixed types — specify --type explicitly.")
            info(f"  Detected: {', '.join(t.value for t in detected)}")
            raise Exit(1)
        pt = detected.pop()
        label = type_label(pt)
        info(f"Detected type: [cyan]{pt.value}[/]  →  [dim]{label}[/]")

        if not yes:
            resp = input(f"  Create as [cyan]{pt.value}[/]? ([Y]/n/custom): ").strip().lower()
            if resp == "n":
                info("Cancelled.")
                raise Exit(0)
            elif resp and resp not in ("y", ""):
                # custom type
                try:
                    pt = PkgType(resp)
                except ValueError:
                    error(f"Unknown type: {resp}")
                    raise Exit(1)
                info(f"Using type: [cyan]{pt.value}[/]")

    # 3 ─ determine app name
    if app_name is None:
        default = resolved[0].stem or resolved[0].name
        if not yes:
            resp = input(f"  App name [[cyan]{default}[/]]: ").strip()
            app_name = resp if resp else default
        else:
            app_name = default

    # 4 ─ destination root
    dest_root = config.DOTFILES_DIR / f"{app_name}{pt.suffix}"

    if dest_root.exists():
        error(f"Package already exists: {dest_root}")
        raise Exit(1)

    # 5 ─ validate source
    for src in resolved:
        _validate_source(src, pt)

    # 6 ─ copy / move files
    rule(f"Creating {pt.value} package: {dest_root.name}")

    for src in resolved:
        dest = _compute_dest(src, pt, dest_root)
        dest.parent.mkdir(parents=True, exist_ok=True)

        if dry_run:
            action = "Move" if pt.is_linux_config else "Copy"
            dim(f"  DRY-RUN  {action}  {src}  →  {dest}")
        else:
            if pt.is_linux_config and pt != PkgType.META:
                shutil.move(str(src), str(dest))
                success(f"Moved   {src.name}  →  {dest}")
            else:
                if src.is_dir():
                    shutil.copytree(str(src), str(dest), dirs_exist_ok=True)
                else:
                    shutil.copy2(str(src), str(dest))
                success(f"Copied  {src.name}  →  {dest}")

    # 7 ─ post-create: stow or sync
    if pt.uses_stow and not no_stow:
        do_stow(dest_root, pt.sync_target, sudo=pt.needs_sudo, dry_run=dry_run)
    elif pt.uses_copy_sync and not no_sync:
        items = prepare_sync_items(dest_root, pt)
        for wsl_p, win_p in items:
            sync_one_file(wsl_p, win_p, direction="to-windows", dry_run=dry_run)

    info("")
    success(f"Package '{dest_root.name}' created.")


def _validate_source(src: Path, pt: PkgType):
    """Raise Exit if source is invalid for the given type."""
    from .display import error
    if pt.is_linux_config and pt != PkgType.META:
        try:
            src.relative_to(Path.home())
        except ValueError:
            error(f"Source must be under $HOME for {pt.value} type: {src}")
            raise Exit(1)
    elif pt.is_windows and pt != PkgType.MNT:
        try:
            src.relative_to(config.MNT_C)
        except ValueError:
            error(f"Source must be under /mnt/c for {pt.value} type: {src}")
            raise Exit(1)


def _compute_dest(src: Path, pt: PkgType, dest_root: Path) -> Path:
    """Compute destination path inside the package."""
    if pt.is_windows:
        # Strip the Windows target prefix from src (/mnt/c path)
        return build_winuser_wsl_path(dest_root, src, pt)

    if pt == PkgType.META:
        return dest_root / src.name

    # Linux types: strip HOME or ROOT prefix
    root = Path.home() if pt.is_linux_config else Path("/")
    try:
        return dest_root / src.relative_to(root)
    except ValueError:
        return dest_root / src.name


# ═══════════════════════════════════════════════════════════════════════════════
#  Command: sync
# ═══════════════════════════════════════════════════════════════════════════════

def cmd_sync(
    pkg_type_str: Optional[str] = None,
    *,
    direction: str = "sync",
    app: Optional[str] = None,
    dry_run: bool = False,
    verbose: bool = False,
    bypass: bool = False,
    quiet: bool = False,
):
    """Sync packages to their targets."""
    from .display import info, success, error, warning, rule, dim
    from .types import type_label, PkgType as PT

    packages = find_packages()
    types_to_sync = list(PkgType) if pkg_type_str is None else [PkgType(pkg_type_str)]

    # Root safety: require explicit confirmation
    has_root = any(pt == PkgType.ROOT for pt in types_to_sync)
    if has_root and not bypass and not dry_run:
        info(f"\n[bold yellow]⚠ Root sync requires sudo and will modify system files.[/]")
        resp = input("  Continue? [y/N]: ").strip().lower()
        if resp != "y":
            info("Cancelled.")
            return

    for pt in types_to_sync:
        pkgs = packages.get(pt, [])
        if app:
            pkgs = [p for p in pkgs if pkg_basename(p) == app]
        if not pkgs:
            continue

        rule(f"Syncing {pt.value} packages")

        if pt.uses_stow:
            if not has_stow():
                error("GNU Stow not installed — skipping.")
                continue
            for pkg in pkgs:
                if not quiet:
                    label = type_label(pt)
                    info(f"  {pkg.name}  →  [dim]{label}[/]")
                do_stow(pkg, pt.sync_target, sudo=pt.needs_sudo, dry_run=dry_run)

        elif pt.uses_copy_sync:
            if not is_wsl():
                warning("Not running in WSL — skipping.")
                continue
            if not has_pwsh():
                error("pwsh.exe not found.")
                continue

            for pkg in pkgs:
                items = prepare_sync_items(pkg, pt)
                if not items:
                    info(f"  {pkg.name}: no files")
                    continue
                info(f"  {pkg.name}: {len(items)} file(s)")

                if not quiet:
                    # Show first few WSL→Windows mappings
                    for wsl_p, win_p in items[:3]:
                        win_str = str(Path("C:/") / win_p.relative_to(config.MNT_C)).replace("/", "\\")
                        dim(f"    {wsl_p.relative_to(pkg)}  →  [cyan]{win_str}[/]")
                    if len(items) > 3:
                        dim(f"    ... and {len(items)-3} more")

                last_pct = 0
                def _progress(result: str, done: int, total: int):
                    nonlocal last_pct
                    pct = done * 100 // total
                    if pct != last_pct:
                        last_pct = pct
                        if not quiet:
                            print(f"\r    [{done}/{total}]" + (" " * 10), end="", flush=True)

                counts = sync_batch(items, direction=direction, dry_run=dry_run, progress_cb=_progress)
                if not quiet:
                    print()
                _print_sync_summary(counts, quiet=quiet)

        else:
            info("  (meta packages are manually managed)")

    info("")
    success("Sync complete.")


def _print_sync_summary(counts: Dict[str, int], *, quiet: bool = False):
    from .display import info
    parts = []
    if counts.get("copied_to_win"): parts.append(f"{counts['copied_to_win']}→win")
    if counts.get("copied_to_wsl"): parts.append(f"{counts['copied_to_wsl']}→wsl")
    if counts.get("up_to_date"):    parts.append(f"{counts['up_to_date']} ok")
    if counts.get("skipped"):       parts.append(f"{counts['skipped']} skipped")
    if counts.get("error"):         parts.append(f"{counts['error']} errors")
    if parts and not quiet:
        info(f"    Result: {', '.join(parts)}")


# ═══════════════════════════════════════════════════════════════════════════════
#  Command: stats
# ═══════════════════════════════════════════════════════════════════════════════

def cmd_stats(*, json_output: bool = False, verbose: bool = False):
    from .display import render_stats
    from .status import check_copy_status, status_text as _st

    packages = find_packages()
    stats_data: Dict[str, dict] = {}
    total_pkgs = total_files = total_bytes = 0

    for pt in PkgType:
        pkgs = packages[pt]
        n_pkgs = len(pkgs)
        n_files = sum(count_files(p) for p in pkgs)
        n_bytes = sum(dir_size(p) for p in pkgs)
        total_pkgs += n_pkgs; total_files += n_files; total_bytes += n_bytes

        if pt.uses_stow:
            stowed = stowable = 0
            for p in pkgs:
                s, t = check_stow_status(p, pt)
                stowed += s; stowable += t
            st = f"{stowed}/{stowable} stowed" if stowable else "empty"
        elif pt.uses_copy_sync:
            tc = {"synced":0,"outdated_local":0,"outdated_remote":0,
                  "missing_remote":0,"missing_local":0,"skipped":0,"error":0}
            for p in pkgs:
                c = check_copy_status(p, pt)
                for k in tc: tc[k] += c.get(k, 0)
            st = _st(tc)
        else:
            st = "manual"

        stats_data[pt.value] = {
            "packages": n_pkgs, "files": n_files,
            "size_bytes": n_bytes, "size_human": fmt_size(n_bytes),
            "status_text": st,
            "names": [pkg_basename(p) for p in pkgs],
        }

    if json_output:
        out = {
            "dotfiles": str(config.DOTFILES_DIR),
            "total_packages": total_pkgs, "total_files": total_files,
            "total_size_bytes": total_bytes, "total_size_human": fmt_size(total_bytes),
            "by_type": {k: {kk: vv for kk, vv in v.items() if kk != "names"}
                        for k, v in stats_data.items()},
        }
        print(json.dumps(out, indent=2, ensure_ascii=False))
        return

    render_stats(stats_data, total_pkgs, total_files, total_bytes)


# ═══════════════════════════════════════════════════════════════════════════════
#  Command: list
# ═══════════════════════════════════════════════════════════════════════════════

def cmd_list(
    pkg_type_str: Optional[str] = None,
    *, json_output: bool = False, verbose: bool = False,
    unsynced: bool = False,
):
    from .display import render_list, warning, info as _info
    from .status import check_copy_status, status_text as _st

    packages = find_packages()
    types_to_show = list(PkgType) if pkg_type_str is None else [PkgType(pkg_type_str)]
    rows: List[Dict] = []

    for pt in types_to_show:
        for pkg in packages.get(pt, []):
            name = pkg_basename(pkg)
            files = count_files(pkg)
            size = dir_size(pkg)
            if pt.uses_stow:
                s, t = check_stow_status(pkg, pt)
                st = "stowed" if (s == t and t > 0) else f"{s}/{t} stowed" if t > 0 else "empty"
            elif pt.uses_copy_sync:
                st = _st(check_copy_status(pkg, pt))
            else:
                st = "manual"

            # --unsynced filter: skip fully-synced packages
            if unsynced:
                if st in ("stowed", "empty", "manual"):
                    continue
                if "synced" in str(st) and "missing" not in str(st) and "outdated" not in str(st):
                    continue
                if "stowed" in str(st) and "/" in str(st):
                    parts = str(st).split("/")
                    if parts[0] == parts[1].split()[0]:
                        continue

            rows.append({
                "name": name, "type": pt.value, "files": files,
                "size_bytes": size, "size_human": fmt_size(size),
                "status": st, "path": str(pkg),
            })

    if json_output:
        print(json.dumps(rows, indent=2, ensure_ascii=False))
        return
    if not rows:
        warning("No packages found."); return
    render_list(rows)

    # --unsynced: show per-package file differences
    if unsynced:
        _info("\n[bold]File differences:[/]")
        for pt in types_to_show:
            for pkg in packages.get(pt, []):
                if not pt.uses_copy_sync and not pt.uses_stow:
                    continue
                _show_diff(pkg, pt)


def _show_diff(pkg: Path, pt: PkgType):
    """Print per-file sync differences for a package."""
    from .display import info, warning, dim
    from .status import check_copy_status, _is_symlink
    from .discover import list_syncable_files, build_mnt_path

    if pt.uses_stow:
        # Show which files are not symlinked
        target = pt.sync_target
        if target:
            missing = []
            for f in pkg.rglob("*"):
                if not f.is_file() or ".git" in f.parts:
                    continue
                dest = target / f.relative_to(pkg)
                try:
                    linked = _is_symlink(dest) and (dest.exists() or _is_symlink(dest))
                    if not linked:
                        p = dest.parent
                        while p != target and p != p.parent:
                            try:
                                if _is_symlink(p) and (p.exists() or _is_symlink(p)):
                                    linked = True
                                    break
                            except PermissionError:
                                pass
                            p = p.parent
                    if not linked:
                        missing.append(str(dest))
                except PermissionError:
                    pass
            if missing:
                warning(f"  {pkg.name} — {len(missing)} file(s) not stowed:")
                for m in missing[:20]:
                    dim(f"    {m}")
                if len(missing) > 20:
                    dim(f"    ... and {len(missing)-20} more")
    elif pt.uses_copy_sync:
        counts = check_copy_status(pkg, pt)
        if counts.get("outdated_local") or counts.get("outdated_remote") or counts.get("missing_remote"):
            warning(f"  {pkg.name} — {status_text(counts)}")
            # List specific outdated files
            files = list_syncable_files(pkg, pt)
            for f in files:
                win_path = build_mnt_path(f, pt)
                try:
                    ws = f.stat()
                    try:
                        wn = win_path.stat()
                    except OSError:
                        wn = None
                except OSError:
                    continue
                if wn is None:
                    dim(f"    missing-win: {f.relative_to(pkg)}")
                elif abs(ws.st_mtime - wn.st_mtime) >= 1.0 or ws.st_size != wn.st_size:
                    newer = "local" if ws.st_mtime > wn.st_mtime else "remote"
                    dim(f"    {newer}-newer: {f.relative_to(pkg)}")


# ═══════════════════════════════════════════════════════════════════════════════
#  CLI builders
# ═══════════════════════════════════════════════════════════════════════════════

def _build_typer_app():
    app = typer.Typer(
        name="wots",
        help="WSL Dotfile Stow Tool — unified dotfile management.",
        no_args_is_help=True,
        pretty_exceptions_show_locals=False,
        context_settings={"help_option_names": ["-h", "--help"]},
    )

    type_help = "Type: " + ", ".join(t.value for t in PkgType)

    @app.command()
    def create(
        sources: List[str] = typer.Argument(..., help="Source file(s) or dir(s)."),
        app_name: Optional[str] = typer.Option(None, "--app-name", "-a", help="Custom app name."),
        pkg_type: Optional[str] = typer.Option(None, "--type", "-t", help=type_help),
        no_stow: bool = typer.Option(False, "--no-stow", help="Skip stow after creation."),
        no_sync: bool = typer.Option(False, "--no-sync", help="Skip Windows sync after creation."),
        dry_run: bool = typer.Option(False, "--dry-run", "-n", help="Preview only."),
        yes: bool = typer.Option(False, "--yes", "-y", help="Skip confirmation prompts."),
    ):
        """Create a new package from existing config files.

        Type is auto-detected from path.  Interactive unless --yes.

        Linux:   ~/.config/* → config,  ~/* → user,  /etc/* → root
        Windows: /mnt/c/Users/{u}/AppData/Roaming/* → winroaming,
                 /mnt/c/Users/{u}/.config/* → winconfig,
                 /mnt/c/Users/{u}/* → winuser
        """
        cmd_create(sources, app_name, pkg_type,
                   no_stow=no_stow, no_sync=no_sync, dry_run=dry_run, yes=yes)

    @app.command()
    def sync(
        pkg_type: Optional[str] = typer.Option(None, "--type", "-t", help=type_help),
        direction: str = typer.Option("sync", "--direction", "-d",
                                      help="Copy direction: sync, to-windows, to-wsl."),
        app: Optional[str] = typer.Option(None, "--app", help="Sync only a specific package."),
        dry_run: bool = typer.Option(False, "--dry-run", "-n", help="Preview only."),
        bypass: bool = typer.Option(False, "--bypass", help="Skip root confirmation."),
        quiet: bool = typer.Option(False, "--quiet", "-q", help="Minimal output."),
    ):
        """Sync packages to their targets.  Stow for Linux, copy-sync for Windows."""
        cmd_sync(pkg_type, direction=direction, app=app,
                 dry_run=dry_run, bypass=bypass, quiet=quiet)

    @app.command()
    def stats(
        json_output: bool = typer.Option(False, "--json", "-j", help="JSON output."),
    ):
        """Repository statistics: packages, files, sizes, sync status."""
        cmd_stats(json_output=json_output)

    @app.command()
    def list(
        pkg_type: Optional[str] = typer.Option(None, "--type", "-t", help=type_help),
        json_output: bool = typer.Option(False, "--json", "-j", help="JSON output."),
        unsynced: bool = typer.Option(False, "--unsynced", "-u", help="Show unsynced packages with file diffs."),
    ):
        """List all packages: name, type, files, size, status."""
        cmd_list(pkg_type, json_output=json_output,
                 unsynced=unsynced)

    return app


def _build_argparse_parser():
    import argparse
    tc = [t.value for t in PkgType]
    p = argparse.ArgumentParser(prog="wots", description="WSL Dotfile Stow Tool.",
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="command", title="commands")

    sp = sub.add_parser("create", help="Create a new package")
    sp.add_argument("sources", nargs="+")
    sp.add_argument("-a", "--app-name", default=None)
    sp.add_argument("-t", "--type", dest="pkg_type", choices=tc)
    sp.add_argument("--no-stow", action="store_true")
    sp.add_argument("--no-sync", action="store_true")
    sp.add_argument("-n", "--dry-run", action="store_true")
    sp.add_argument("-y", "--yes", action="store_true")

    sp = sub.add_parser("sync", help="Sync packages")
    sp.add_argument("-t", "--type", dest="pkg_type", choices=tc)
    sp.add_argument("-d", "--direction", default="sync")
    sp.add_argument("--app", default=None)
    sp.add_argument("-n", "--dry-run", action="store_true")
    sp.add_argument("-v", "--verbose", action="store_true")
    sp.add_argument("--bypass", action="store_true")
    sp.add_argument("-q", "--quiet", action="store_true")

    sp = sub.add_parser("stats", help="Statistics")
    sp.add_argument("-j", "--json", dest="json_output", action="store_true")
    
    sp.add_argument("-t", "--type", dest="pkg_type", choices=tc)
    sp.add_argument("-j", "--json", dest="json_output", action="store_true")
    sp.add_argument("-v", "--verbose", action="store_true")
    sp.add_argument("-u", "--unsynced", action="store_true")
    sp.add_argument("--diff", action="store_true")

    return p


def main():
    if HAS_TYPER:
        _build_typer_app()()
    else:
        args = _build_argparse_parser().parse_args()
        if args.command is None:
            _build_argparse_parser().print_help(); sys.exit(0)
        if args.command == "create":
            cmd_create(args.sources, args.app_name, args.pkg_type,
                       no_stow=args.no_stow, no_sync=args.no_sync,
                       dry_run=args.dry_run, yes=getattr(args, 'yes', False))
        elif args.command == "sync":
            cmd_sync(args.pkg_type, direction=args.direction, app=args.app,
                     dry_run=args.dry_run, verbose=args.verbose,
                     bypass=getattr(args, 'bypass', False),
                     quiet=getattr(args, 'quiet', False))
        elif args.command == "stats":
            cmd_stats(json_output=args.json_output)
        elif args.command == "list":
            cmd_list(args.pkg_type, json_output=args.json_output, verbose=args.verbose,
                     unsynced=getattr(args, 'unsynced', False),
                     diff=getattr(args, 'diff', False))


if __name__ == "__main__":
    main()
