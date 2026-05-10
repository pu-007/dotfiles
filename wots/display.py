"""
Rich / plain-text display helpers.

When Rich is available the output uses colours, panels, and tables.
Otherwise it degrades gracefully to plain text.
"""
from __future__ import annotations

import sys
import re
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from . import config

# ── Rich detection ─────────────────────────────────────────────

HAS_RICH = False
_RC = None

try:
    from rich.console import Console
    from rich.table import Table
    from rich.panel import Panel
    from rich.progress import (
        Progress, SpinnerColumn, TextColumn,
        BarColumn, TaskProgressColumn, TimeRemainingColumn,
    )
    from rich.live import Live
    from rich.text import Text
    HAS_RICH = True
    _RC = Console(highlight=False)
except ImportError:
    pass


# ── basic print helpers ───────────────────────────────────────

def info(msg: str = ""):
    if HAS_RICH:
        _RC.print(msg)
    else:
        print(msg)

def success(msg: str):
    if HAS_RICH:
        _RC.print(f"[green]✓[/] {msg}")
    else:
        print(f"✓ {msg}")

def error(msg: str):
    if HAS_RICH:
        _RC.print(f"[red]✗[/] {msg}")
    else:
        print(f"✗ {msg}", file=sys.stderr)

def warning(msg: str):
    if HAS_RICH:
        _RC.print(f"[yellow]![/] {msg}")
    else:
        print(f"! {msg}")

def dim(msg: str):
    if HAS_RICH:
        _RC.print(f"[dim]{msg}[/]")
    else:
        print(msg)

def rule(title: str = ""):
    if HAS_RICH:
        _RC.rule(title)
    else:
        if title:
            print(f"\n── {title} ──")
        else:
            print("─" * 50)


# ── stats table ───────────────────────────────────────────────

def render_stats(
    stats_data: Dict[str, Any],
    total_pkgs: int,
    total_files: int,
    total_bytes: int,
):
    """Render the repository statistics table."""
    from .utils import fmt_size
    from .types import PkgType

    if HAS_RICH:
        table = Table(title="WOTS Repository Statistics")
        table.add_column("Type", style="cyan")
        table.add_column("Pkgs", justify="right")
        table.add_column("Files", justify="right")
        table.add_column("Size", justify="right")
        table.add_column("Status")

        for pt in PkgType:
            d = stats_data.get(pt.value, {})
            if d.get("packages", 0) == 0:
                continue
            status = d.get("status_text", "—")
            table.add_row(
                pt.value, str(d["packages"]), str(d["files"]),
                d.get("size_human", "0 B"), status,
            )

        table.add_row(
            "[bold]TOTAL[/]", f"[bold]{total_pkgs}[/]",
            f"[bold]{total_files}[/]", f"[bold]{fmt_size(total_bytes)}[/]",
            "", style="bold",
        )
        _RC.print(table)
    else:
        print(f"WOTS Repository  —  {config.DOTFILES_DIR}\n")
        print(f"{'Type':<10} {'Pkgs':>6} {'Files':>7} {'Size':>10}  Status")
        print("-" * 60)
        for pt in PkgType:
            d = stats_data.get(pt.value, {})
            if d.get("packages", 0) == 0:
                continue
            print(
                f"{pt.value:<10} {d['packages']:>6} {d['files']:>7} "
                f"{d.get('size_human', '0 B'):>10}  {d.get('status_text', '—')}"
            )
        print("-" * 60)
        print(f"{'TOTAL':<10} {total_pkgs:>6} {total_files:>7} {fmt_size(total_bytes):>10}")


# ── list table ────────────────────────────────────────────────

def render_list(rows: List[Dict[str, Any]]):
    """Render the package listing table."""
    if HAS_RICH:
        table = Table(title="Dotfile Packages")
        table.add_column("Package", style="cyan bold")
        table.add_column("Type", style="green")
        table.add_column("Files", justify="right")
        table.add_column("Size", justify="right")
        table.add_column("Status")
        for r in rows:
            table.add_row(
                r["name"], r["type"], str(r["files"]),
                r.get("size_human", "0 B"), r.get("status", "—"),
            )
        _RC.print(table)
        info(f"\n{len(rows)} package(s) total.")
    else:
        print(f"{'Package':<24} {'Type':<8} {'Files':>6} {'Size':>10}  Status")
        print("-" * 68)
        for r in rows:
            print(
                f"{r['name']:<24} {r['type']:<8} {r['files']:>6} "
                f"{r.get('size_human', '0 B'):>10}  {r.get('status', '—')}"
            )
        print(f"\n{len(rows)} package(s) total.")


# ── progress ──────────────────────────────────────────────────

def create_progress() -> Any:
    """Create a Rich Progress context manager, or a no-op."""
    if HAS_RICH:
        return Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TaskProgressColumn(),
            TimeRemainingColumn(),
            console=_RC,
        )
    else:
        return _PlainProgress()

class _PlainProgress:
    """Minimal progress fallback when Rich is unavailable."""
    def __init__(self):
        self._task = None
        self._total = 0

    def __enter__(self):
        return self

    def __exit__(self, *args):
        pass

    def add_task(self, description: str, total: int, **kw):
        self._total = total
        print(f"\n{description} ...")
        return 0

    def update(self, task_id, advance: int = 1, description: str = ""):
        pass
