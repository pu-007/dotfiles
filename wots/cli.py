"""
CLI — typer app with argparse fallback.

Commands:
  create   Create a new stow/winuser package from existing files
  sync     Sync (stow / copy) packages to their targets
  stats    Show repository statistics
  list     List all packages with details
"""
from __future__ import annotations

import sys
import json
import shutil
from pathlib import Path
from typing import Optional, List, Dict, Tuple

from . import config
from .types import PkgType
from .discover import detect_type, find_packages, pkg_basename, list_syncable_files
from .sync import (
    do_stow, sync_one_file, sync_batch, prepare_sync_items, build_winuser_wsl_path,
)
from .status import check_stow_status, check_copy_status, status_text
from .utils import (
    count_files, dir_size, fmt_size, is_wsl, has_pwsh, has_stow, is_excluded,
)

# ── Try typer, fall back to argparse ──────────────────────────

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

    # 2 ─ determine type
    if pkg_type_str:
        try:
            pt = PkgType(pkg_type_str)
        except ValueError:
            error(f"Unknown type: {pkg_type_str}.  Use: user, root, winuser, meta, mnt")
            raise Exit(1)
    else:
        detected = {detect_type(p) for p in resolved}
        if len(detected) > 1:
            error("Sources have mixed types — specify --type explicitly.")
            info(f"  Detected: {', '.join(t.value for t in detected)}")
            raise Exit(1)
        pt = detected.pop()
        info(f"Auto-detected type: [cyan]{pt.value}[/]")

    # 3 ─ determine app name
    if pt in (PkgType.MNT,):
        app_name = None
    elif app_name is None:
        app_name = resolved[0].stem or resolved[0].name

    # 4 ─ destination
    if pt == PkgType.MNT:
        dest_root = config.WSL_MNT_BASE
    elif pt == PkgType.WINUSER:
        dest_root = config.DOTFILES_DIR / f"{app_name}{pt.suffix}"
    else:
        dest_root = config.DOTFILES_DIR / f"{app_name}{pt.suffix}"

    if dest_root.exists() and pt != PkgType.MNT:
        error(f"Package already exists: {dest_root}")
        info("  Remove it first or choose a different --app-name.")
        raise Exit(1)

    # 5 ─ validate source root
    for src in resolved:
        if pt == PkgType.USER:
            try:
                src.relative_to(Path.home())
            except ValueError:
                error(f"USER source must be under $HOME: {src}")
                raise Exit(1)
        elif pt in (PkgType.WINUSER, PkgType.MNT):
            try:
                src.relative_to(config.MNT_C)
            except ValueError:
                error(f"{pt.value.upper()} source must be under /mnt/c: {src}")
                raise Exit(1)

    # 6 ─ copy / move
    rule(f"Creating {pt.value} package: {dest_root.name}")

    for src in resolved:
        if pt == PkgType.MNT:
            rel = src.relative_to(config.MNT_C)
            dest = dest_root / rel
        elif pt == PkgType.WINUSER:
            try:
                rel = src.relative_to(config.MNT_C / "Users" / config.WIN_USERNAME)
            except ValueError:
                rel = src.relative_to(config.MNT_C)
            dest = dest_root / config.WIN_USERNAME / rel
        elif pt == PkgType.META:
            dest = dest_root / src.name
        else:
            root = Path.home() if pt == PkgType.USER else Path("/")
            try:
                rel = src.relative_to(root)
            except ValueError:
                rel = Path(src.name)
            dest = dest_root / rel

        dest.parent.mkdir(parents=True, exist_ok=True)

        if dry_run:
            action = "Move" if pt in (PkgType.USER, PkgType.ROOT) else "Copy"
            dim(f"  DRY-RUN  {action}  {src}  →  {dest}")
        else:
            if pt in (PkgType.USER, PkgType.ROOT):
                shutil.move(str(src), str(dest))
                success(f"Moved   {src.name}  →  {dest}")
            else:
                if src.is_dir():
                    shutil.copytree(str(src), str(dest), dirs_exist_ok=True)
                else:
                    shutil.copy2(str(src), str(dest))
                success(f"Copied  {src.name}  →  {dest}")

    # 7 ─ post-create
    if pt.uses_stow and not no_stow:
        do_stow(dest_root, pt.stow_target, sudo=pt.needs_sudo, dry_run=dry_run)
    elif pt.uses_copy_sync and not no_sync:
        items = prepare_sync_items(dest_root, pt)
        if items:
            direction = "to-windows" if pt in (PkgType.WINUSER, PkgType.MNT) else "sync"
            for wsl_p, win_p in items:
                sync_one_file(wsl_p, win_p, direction=direction, dry_run=dry_run)

    info("")
    success(f"Package '{dest_root.name}' created.")


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
):
    """Sync packages to their targets."""
    from .display import info, success, error, warning, rule, create_progress

    packages = find_packages()
    types_to_sync = list(PkgType) if pkg_type_str is None else [PkgType(pkg_type_str)]

    for pt in types_to_sync:
        pkgs = packages.get(pt, [])
        if app:
            pkgs = [p for p in pkgs if pkg_basename(p) == app]
        if not pkgs:
            continue

        rule(f"Syncing {pt.value} packages")

        if pt.uses_stow:
            if not has_stow():
                error("GNU Stow not installed — skipping stow.")
                continue
            for pkg in pkgs:
                do_stow(pkg, pt.stow_target, sudo=pt.needs_sudo, dry_run=dry_run)

        elif pt.uses_copy_sync:
            if not is_wsl():
                warning("Not running in WSL — skipping copy-sync.")
                continue
            if not has_pwsh():
                error("pwsh.exe not found — cannot sync Windows files.")
                continue

            for pkg in pkgs:
                items = prepare_sync_items(pkg, pt)
                if not items:
                    info(f"  No files to sync in {pkg.name}")
                    continue

                info(f"  {pkg.name}: {len(items)} file(s)")

                # Progress callback
                last_pct = 0
                def _progress(result: str, done: int, total: int):
                    nonlocal last_pct
                    pct = done * 100 // total
                    if verbose or pct != last_pct:
                        last_pct = pct
                        if verbose:
                            info(f"    [{done}/{total}] {result}")
                        else:
                            # Overwrite same line
                            print(f"\r    [{done}/{total}] synced", end="", flush=True)

                counts = sync_batch(
                    items, direction=direction, dry_run=dry_run,
                    progress_cb=_progress,
                )
                if not verbose:
                    print()  # newline after progress
                _print_sync_summary(counts)

        else:  # META
            info("  (meta packages are manually managed — nothing to sync)")

    info("")
    success("Sync complete.")


def _print_sync_summary(counts: Dict[str, int]):
    """Print a one-line summary of sync results."""
    from .display import info, success, warning, error as err
    parts = []
    if counts.get("copied_to_win"):
        parts.append(f"{counts['copied_to_win']}→win")
    if counts.get("copied_to_wsl"):
        parts.append(f"{counts['copied_to_wsl']}→wsl")
    if counts.get("up_to_date"):
        parts.append(f"{counts['up_to_date']} ok")
    if counts.get("skipped"):
        parts.append(f"{counts['skipped']} skipped")
    if counts.get("error"):
        parts.append(f"{counts['error']} errors")
    if parts:
        info(f"    Result: {', '.join(parts)}")


# ═══════════════════════════════════════════════════════════════════════════════
#  Command: stats
# ═══════════════════════════════════════════════════════════════════════════════

def cmd_stats(*, json_output: bool = False, verbose: bool = False):
    """Show repository statistics."""
    from .display import render_stats
    from .status import check_copy_status, status_text as _status_text

    packages = find_packages()
    stats_data: Dict[str, dict] = {}
    total_pkgs = 0
    total_files = 0
    total_bytes = 0

    for pt in PkgType:
        pkgs = packages[pt]
        n_pkgs = len(pkgs)
        n_files = sum(count_files(p) for p in pkgs)
        n_bytes = sum(dir_size(p) for p in pkgs)
        total_pkgs += n_pkgs
        total_files += n_files
        total_bytes += n_bytes

        if pt.uses_stow:
            stowed = 0
            stowable = 0
            for p in pkgs:
                s, t = check_stow_status(p, pt)
                stowed += s
                stowable += t
            status_txt = f"{stowed}/{stowable} stowed" if stowable else "empty"
        elif pt.uses_copy_sync:
            total_counts = {
                "synced": 0, "outdated_local": 0, "outdated_remote": 0,
                "missing_remote": 0, "missing_local": 0, "skipped": 0, "error": 0,
            }
            for p in pkgs:
                c = check_copy_status(p, pt)
                for k in total_counts:
                    total_counts[k] += c.get(k, 0)
            status_txt = _status_text(total_counts)
        else:
            status_txt = "manual only"

        stats_data[pt.value] = {
            "packages": n_pkgs,
            "files": n_files,
            "size_bytes": n_bytes,
            "size_human": fmt_size(n_bytes),
            "status_text": status_txt,
            "names": [pkg_basename(p) for p in pkgs],
        }

    if json_output:
        print(json.dumps({
            "dotfiles": str(config.DOTFILES_DIR),
            "total_packages": total_pkgs,
            "total_files": total_files,
            "total_size_bytes": total_bytes,
            "total_size_human": fmt_size(total_bytes),
            "by_type": {k: {kk: vv for kk, vv in v.items() if kk != "names"}
                        for k, v in stats_data.items()},
        }, indent=2, ensure_ascii=False))
        return

    render_stats(stats_data, total_pkgs, total_files, total_bytes)


# ═══════════════════════════════════════════════════════════════════════════════
#  Command: list
# ═══════════════════════════════════════════════════════════════════════════════

def cmd_list(
    pkg_type_str: Optional[str] = None,
    *,
    json_output: bool = False,
    verbose: bool = False,
):
    """List all packages with details."""
    from .display import render_list
    from .status import check_copy_status, status_text as _status_text

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
                status = "stowed" if (s == t and t > 0) else f"{s}/{t} stowed" if t > 0 else "empty"
            elif pt.uses_copy_sync:
                counts = check_copy_status(pkg, pt)
                status = _status_text(counts)
            else:
                status = "manual"

            rows.append({
                "name": name,
                "type": pt.value,
                "files": files,
                "size_bytes": size,
                "size_human": fmt_size(size),
                "status": status,
                "path": str(pkg),
            })

    if json_output:
        print(json.dumps(rows, indent=2, ensure_ascii=False))
        return

    if not rows:
        from .display import warning
        warning("No packages found.")
        return

    render_list(rows)


# ═══════════════════════════════════════════════════════════════════════════════
#  CLI builders
# ═══════════════════════════════════════════════════════════════════════════════

def _build_typer_app() -> "typer.Typer":
    app = typer.Typer(
        name="wots",
        help="WSL Dotfile Stow Tool — unified dotfile management.",
        no_args_is_help=True,
        pretty_exceptions_show_locals=False,
    )

    # ── create ──
    @app.command()
    def create(
        sources: List[str] = typer.Argument(..., help="Source file(s) or directory(ies) to package."),
        app_name: Optional[str] = typer.Option(None, "--app-name", "-a", help="Custom app name."),
        pkg_type: Optional[str] = typer.Option(None, "--type", "-t", help="Type: user, root, winuser, meta, mnt."),
        no_stow: bool = typer.Option(False, "--no-stow", help="Skip stow after creation."),
        no_sync: bool = typer.Option(False, "--no-sync", help="Skip Windows sync after creation."),
        dry_run: bool = typer.Option(False, "--dry-run", "-n", help="Preview only."),
    ):
        """Create a new package from existing config files.

        Auto-detects type:
          $HOME/*                    → user
          /mnt/c/Users/{user}/*      → winuser
          /mnt/c/*                   → mnt
          /etc/*                     → root
          other                      → meta
        """
        cmd_create(sources, app_name, pkg_type,
                   no_stow=no_stow, no_sync=no_sync, dry_run=dry_run)

    # ── sync ──
    @app.command()
    def sync(
        pkg_type: Optional[str] = typer.Option(None, "--type", "-t", help="Sync only: user, root, winuser, meta, mnt."),
        direction: str = typer.Option("sync", "--direction", "-d", help="Copy direction: sync, to-windows, to-wsl."),
        app: Optional[str] = typer.Option(None, "--app", help="Sync only a specific package by name."),
        dry_run: bool = typer.Option(False, "--dry-run", "-n", help="Preview only."),
        verbose: bool = typer.Option(False, "--verbose", "-v", help="Show per-file details."),
    ):
        """Sync (stow / copy) packages to their targets.

        user/root  → stow to $HOME or /
        winuser    → copy-sync to C:\\Users\\...
        mnt        → copy-sync c.mnt/ ↔ C:\\
        meta       → skipped (manual)
        """
        cmd_sync(pkg_type, direction=direction, app=app,
                 dry_run=dry_run, verbose=verbose)

    # ── stats ──
    @app.command()
    def stats(
        json_output: bool = typer.Option(False, "--json", "-j", help="Machine-readable JSON."),
        verbose: bool = typer.Option(False, "--verbose", "-v", help="Per-package details."),
    ):
        """Repository statistics: packages, files, sizes, sync status."""
        cmd_stats(json_output=json_output, verbose=verbose)

    # ── list ──
    @app.command()
    def list(
        pkg_type: Optional[str] = typer.Option(None, "--type", "-t", help="Filter: user, root, winuser, meta, mnt."),
        json_output: bool = typer.Option(False, "--json", "-j", help="Machine-readable JSON."),
        verbose: bool = typer.Option(False, "--verbose", "-v", help="Show per-file status (winuser/mnt)."),
    ):
        """List all packages: name, type, files, size, status."""
        cmd_list(pkg_type, json_output=json_output, verbose=verbose)

    return app


def _build_argparse_parser():
    import argparse
    parser = argparse.ArgumentParser(
        prog="wots",
        description="WSL Dotfile Stow Tool — unified dotfile management.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""examples:
  wots create ~/.config/nvim
  wots sync
  wots sync --type winuser --direction to-windows
  wots stats
  wots list --type user
        """,
    )
    sub = parser.add_subparsers(dest="command", title="commands")

    # create
    p = sub.add_parser("create", help="Create a new package")
    p.add_argument("sources", nargs="+", help="Source files/dirs")
    p.add_argument("-a", "--app-name", default=None)
    p.add_argument("-t", "--type", dest="pkg_type", default=None,
                   choices=["user", "root", "winuser", "meta", "mnt"])
    p.add_argument("--no-stow", action="store_true")
    p.add_argument("--no-sync", action="store_true")
    p.add_argument("-n", "--dry-run", action="store_true")

    # sync
    p = sub.add_parser("sync", help="Sync packages to targets")
    p.add_argument("-t", "--type", dest="pkg_type",
                   choices=["user", "root", "winuser", "meta", "mnt"])
    p.add_argument("-d", "--direction", default="sync",
                   choices=["sync", "to-windows", "to-wsl"])
    p.add_argument("--app", default=None)
    p.add_argument("-n", "--dry-run", action="store_true")
    p.add_argument("-v", "--verbose", action="store_true")

    # stats
    p = sub.add_parser("stats", help="Repository statistics")
    p.add_argument("-j", "--json", dest="json_output", action="store_true")
    p.add_argument("-v", "--verbose", action="store_true")

    # list
    p = sub.add_parser("list", help="List packages")
    p.add_argument("-t", "--type", dest="pkg_type",
                   choices=["user", "root", "winuser", "meta", "mnt"])
    p.add_argument("-j", "--json", dest="json_output", action="store_true")
    p.add_argument("-v", "--verbose", action="store_true")

    return parser


# ═══════════════════════════════════════════════════════════════════════════════
#  Entry point
# ═══════════════════════════════════════════════════════════════════════════════

def main():
    if HAS_TYPER:
        app = _build_typer_app()
        app()
    else:
        parser = _build_argparse_parser()
        args = parser.parse_args()

        if args.command is None:
            parser.print_help()
            sys.exit(0)

        if args.command == "create":
            cmd_create(args.sources, args.app_name, args.pkg_type,
                       no_stow=args.no_stow, no_sync=args.no_sync, dry_run=args.dry_run)
        elif args.command == "sync":
            cmd_sync(args.pkg_type, direction=args.direction, app=args.app,
                     dry_run=args.dry_run, verbose=args.verbose)
        elif args.command == "stats":
            cmd_stats(json_output=args.json_output, verbose=args.verbose)
        elif args.command == "list":
            cmd_list(args.pkg_type, json_output=args.json_output, verbose=args.verbose)


if __name__ == "__main__":
    main()
