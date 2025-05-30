#!/usr/bin/env python3
# config_linker.py

import subprocess
from pathlib import Path
import argparse
import time

# --- Configuration (can be overridden by CLI args or function params) ---
# Expands to /home/your_wsl_user/dotfiles/c.mnt (or similar)
DEFAULT_WSL_CONFIG_ROOT_STR = str(Path.home() / "dotfiles" / "c.mnt")
DEFAULT_WSL_DISTRO_NAME = "Arch"  # Change if your distro name is different

# --- Helper Functions for Command Execution ---


def _run_wsl_command_helper(command_parts: list, dry_run: bool = False) -> bool:
    """
    Executes a command directly in WSL (e.g., rm -rf).
    Returns True on success, False on failure.
    """
    # command_parts should be a list, e.g., ['rm', '-rf', '/path/to/target']
    print(f"  INTERNAL WSL CMD: {' '.join(command_parts)}")
    if dry_run:
        print("    (DRY_RUN _run_wsl_command_helper) Command not actually executed.")
        return True  # Simulate success for dry run

    try:
        # Using text=True and utf-8 for WSL commands generally.
        result = subprocess.run(
            command_parts,
            capture_output=True,
            text=True,
            check=False,
            encoding="utf-8",
            errors="replace",
        )
        if result.stdout.strip():
            print(f"    STDOUT: {result.stdout.strip()}")
        if result.stderr.strip():
            print(
                f"    STDERR: {result.stderr.strip()}"
            )  # rm -rf usually silent on success

        if result.returncode != 0:
            print(f"    WSL command failed with return code: {result.returncode}")
            return False
        return True
    except Exception as e:
        print(
            f"  Exception while executing WSL command '{' '.join(command_parts)}': {e}"
        )
        return False


def _run_windows_cmd_via_pwsh_helper(
    cmd_command_parts_list: list, dry_run: bool = False
) -> bool:
    """
    Executes a Windows CMD command (like mklink) via PowerShell from WSL.
    cmd_command_parts_list should be a list for cmd, e.g., ['mklink', '/D', '"C:\\Link"', '"\\\\Target"']
    Returns True on success, False on failure.
    """
    # create the parent directory for the command if it doesn't exist
    # cmd /c expects a single command string. Parts are joined.
    cmd_command_str_for_pwsh = " ".join(cmd_command_parts_list)
    # PowerShell command to execute the cmd string
    full_pwsh_command = [
        "pwsh.exe",
        "-NoProfile",
        "-Command",
        f"cmd /c {cmd_command_str_for_pwsh}",
    ]

    print(f"  INTERNAL WIN CMD (via pwsh): {' '.join(full_pwsh_command)}")
    if dry_run:
        print(
            "    (DRY_RUN _run_windows_cmd_via_pwsh_helper) Command not actually executed."
        )
        return True

    try:
        # cmd.exe output is often not UTF-8. 'gbk' is a common console encoding.
        # errors='replace' will prevent crashes if some characters can't be decoded.
        result = subprocess.run(
            full_pwsh_command,
            capture_output=True,
            text=True,
            cwd="/mnt/c",
            encoding="gbk",
            errors="replace",
            check=False,
        )
        # mklink often prints success to stderr or stdout depending on version/language
        if result.stdout.strip():
            print(f"    STDOUT: {result.stdout.strip()}")
        if result.stderr.strip():
            print(f"    STDERR: {result.stderr.strip()}")

        if result.returncode != 0:
            print(f"    Windows command failed with return code: {result.returncode}")
            return False
        # Assuming return code 0 means success for mklink for now
        return True
    except Exception as e:
        print(
            f"  Exception while executing Windows command '{cmd_command_str_for_pwsh}': {e}"
        )
        return False


# --- Core Symlinking Function ---


def create_windows_symlink(
    wsl_item_path_str: str,
    wsl_config_root_str: str,
    wsl_distro_name: str,
    confirm_deletion: bool = True,
    dry_run: bool = False,
) -> bool:
    """
    Creates a Windows symbolic link for a given WSL path.

    The WSL item at wsl_item_path_str (which should be an absolute path or
    resolve correctly from a relative one given to the script) will be linked
    from the corresponding path on the Windows C: drive. The wsl_config_root_str
    defines the base in WSL that maps to "C:\".
    The original item on the C: drive (if any) will be deleted after confirmation (if applicable).

    Args:
        wsl_item_path_str: Absolute path string to the source item in WSL.
        wsl_config_root_str: Absolute path string in WSL that maps to "C:\".
        wsl_distro_name: Name of the WSL distribution.
        confirm_deletion: If True, ask user before deleting existing directories on Windows.
        dry_run: If True, print actions instead of performing them.

    Returns:
        True if successful or skipped by user choice, False on error.
    """
    print(f"\nProcessing WSL item: {wsl_item_path_str}")
    # These parameters are used for context but not directly modified here.
    # print(f"  Using WSL Config Root: {wsl_config_root_str}")
    # print(f"  Using WSL Distro Name: {wsl_distro_name}")
    # print(f"  Deletion Confirmation: {confirm_deletion}")
    # print(f"  Dry Run Mode: {dry_run}")

    wsl_item_path = Path(wsl_item_path_str)  # Already resolved by CLI
    wsl_config_root_path = Path(wsl_config_root_str)  # Already resolved by CLI

    if not wsl_item_path.exists():
        print(f"  ERROR: WSL source item '{wsl_item_path}' does not exist.")
        return False

    try:
        relative_path = wsl_item_path.relative_to(wsl_config_root_path)
    except ValueError:
        print(
            f"  ERROR: WSL item '{wsl_item_path}' is not inside the WSL config root '{wsl_config_root_path}'."
        )
        print("         Cannot determine relative path for C: drive mapping.")
        return False

    print(f"  Relative path for C: drive mapping: {relative_path}")

    # 1. Determine path on /mnt/c/ for potential deletion
    windows_item_via_mnt_c = Path("/mnt/c") / relative_path
    try:
        windows_item_via_mnt_c.parent.mkdir(
            parents=True, exist_ok=True
        )  # Ensure parent directory exists
    except OSError:  # when the parent directory already exists, this will raise an OSError because os does not support detection of WSL symlinks
        ...
    windows_item_via_mnt_c_str = str(windows_item_via_mnt_c)
    print(f"  Equivalent Windows item (via /mnt/c): {windows_item_via_mnt_c_str}")

    # Deletion logic
    path_to_delete_obj = Path(windows_item_via_mnt_c_str)
    try:
        item_exists_on_windows_side = (
            path_to_delete_obj.exists() or path_to_delete_obj.is_symlink()
        )
    except OSError:  # syslink to WSL alread exists in Windows, however, os.stat fails to detect with OSError
        item_exists_on_windows_side = True

    if item_exists_on_windows_side:
        if dry_run:
            print(
                f"    (DRY_RUN) Would target '{windows_item_via_mnt_c_str}' for deletion."
            )
        else:  # Not dry_run and item exists, so consider actual deletion
            if (
                confirm_deletion and path_to_delete_obj.is_dir()
            ):  # Prompt for any directory (real or symlink to dir)
                user_confirm = input(
                    f"    CONFIRM DELETION of directory '{windows_item_via_mnt_c_str}' on /mnt/c? (yes/no): "
                ).lower()
                if user_confirm != "yes":
                    print(
                        f"    User skipped deletion of '{windows_item_via_mnt_c_str}'. Link creation will be skipped for this item."
                    )
                    return True  # User chose to skip, consider this a "successful" outcome for this item.

            # If we are here, it's either:
            # 1. Not a directory needing confirmation OR
            # 2. It was a directory, and user confirmed 'yes' OR
            # 3. confirm_deletion was False.
            # So, proceed with actual deletion attempt.
            # The path_to_delete_obj string is fine as is for rm -rf argument
            if not _run_wsl_command_helper(
                ["rm", "-rf", windows_item_via_mnt_c_str], dry_run=False
            ):  # Actual deletion attempt
                print(f"  ERROR: Deletion of '{windows_item_via_mnt_c_str}' failed.")
                return False  # Hard failure
    else:  # Item does not exist on /mnt/c
        print(
            f"  Original item '{windows_item_via_mnt_c_str}' not found on /mnt/c. No deletion needed."
        )

    # If we've reached here, deletion was successful, skipped by user (function returned True), or not needed.
    # Now, proceed to mklink.

    # 2. Determine paths for mklink
    windows_link_name_on_c_drive = Path("C:/") / relative_path
    # Ensure backslashes for CMD, and quote for mklink command construction
    windows_link_name_str_for_cmd = str(windows_link_name_on_c_drive).replace("/", "\\")

    wsl_item_abs_path_for_unc = str(wsl_item_path).replace(
        "/", "\\"
    )  # wsl_item_path is already absolute & resolved
    wsl_network_target_str = f"\\\\wsl$\\{wsl_distro_name}{wsl_item_abs_path_for_unc}"
    # For modern Windows: f"\\\\wsl.localhost\\{wsl_distro_name}{wsl_item_abs_path_for_unc}" also works

    print(f'  Windows link name to create: "{windows_link_name_str_for_cmd}"')
    print(f'  Windows link target (WSL UNC): "{wsl_network_target_str}"')

    mklink_cmd_parts = ["mklink"]
    if wsl_item_path.is_dir():
        mklink_cmd_parts.append("/D")

    # Paths for mklink must be quoted if they contain spaces.
    # The f-string with \"...\" handles this when creating the command parts.
    mklink_cmd_parts.append(f'"{windows_link_name_str_for_cmd}"')
    mklink_cmd_parts.append(f'"{wsl_network_target_str}"')

    if not _run_windows_cmd_via_pwsh_helper(mklink_cmd_parts, dry_run=dry_run):
        print(f"  ERROR: mklink command failed for '{windows_link_name_str_for_cmd}'.")
        return False

    print(f"  Successfully processed and linked '{wsl_item_path_str}'.")
    return True


# --- CLI ---
def main_cli():
    parser = argparse.ArgumentParser(
        description="Create Windows symlinks pointing to WSL files/directories. "
        "The WSL paths provided should be items within your WSL config root "
        "(e.g., if root is ~/dotfiles/c.mnt, provide paths like ~/dotfiles/c.mnt/Users/...).",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "wsl_paths",
        metavar="WSL_FULL_PATH",
        type=str,
        nargs="+",
        help="One or more full paths in WSL to create links for (e.g., ~/dotfiles/c.mnt/Users/zion/.config).",
    )
    parser.add_argument(
        "--wsl-config-root",
        type=str,
        default=DEFAULT_WSL_CONFIG_ROOT_STR,
        help="The base path in WSL that corresponds to C:\\.",
    )
    parser.add_argument(
        "--distro-name",
        type=str,
        default=DEFAULT_WSL_DISTRO_NAME,
        help="Your WSL distribution name (e.g., Arch, Ubuntu-20.04).",
    )
    parser.add_argument(
        "--no-confirm-deletion",  # Changed flag name for clarity
        action="store_false",
        dest="confirm_deletion",  # confirm_deletion will be True by default
        help="Do not ask for confirmation before deleting existing directories on the Windows side.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be done, without making any changes.",
    )
    parser.add_argument(
        "--ci",
        "--non-interactive",  # Added --ci alias
        action="store_true",
        dest="ci_mode",
        help="Run non-interactively (implies --no-confirm-deletion for individual items and no main prompt). Useful for scripts.",
    )

    args = parser.parse_args()

    # Resolve paths once at the beginning
    try:
        resolved_wsl_config_root = str(
            Path(args.wsl_config_root).expanduser().resolve(strict=True)
        )
    except FileNotFoundError:
        print(f"Error: WSL config root '{args.wsl_config_root}' does not exist.")
        exit(1)

    resolved_wsl_paths = []
    for p_str in args.wsl_paths:
        try:
            resolved_p = str(Path(p_str).expanduser().resolve(strict=True))
            resolved_wsl_paths.append(resolved_p)
        except FileNotFoundError:
            print(
                f"Error: Input WSL path '{p_str}' does not exist. Skipping this path."
            )
            # Optionally, exit(1) if any path is invalid, or just skip. For now, skip.

    if not resolved_wsl_paths:
        print("No valid WSL paths to process after checking existence.")
        exit(0)

    # Determine effective confirmation setting
    effective_confirm_deletion = args.confirm_deletion
    if args.ci_mode:
        effective_confirm_deletion = (
            False  # CI mode implies no interactive confirmation for deletions
        )
        print(
            "Running in CI mode (non-interactive, no confirmations for individual directory deletions)."
        )

    # Initial overall script confirmation if not in dry_run and not in ci_mode
    if not args.dry_run and not args.ci_mode:
        print("\n--- IMPORTANT SCRIPT PREVIEW ---")
        print(
            f"This script will attempt to process {len(resolved_wsl_paths)} WSL item(s)."
        )
        print(
            "For each item, it may delete content on your Windows C: drive (via /mnt/c/)"
        )
        print("and then create a symbolic link from Windows to WSL.")
        print("  WSL paths to process:")
        for rp in resolved_wsl_paths:
            print(f"    - {rp}")
        print(f"  WSL config root (maps to C:\\): {resolved_wsl_config_root}")
        print(f"  WSL Distro: {args.distro_name}")
        if effective_confirm_deletion:
            print(
                "  Confirmation will be asked before deleting existing directories on /mnt/c."
            )
        else:
            print("  Confirmation for deleting existing directories on /mnt/c is OFF.")
        print("---")
        try:
            if effective_confirm_deletion:
                proceed_all = input(
                    "Do you want to proceed with the overall operation? (yes/no): "
                ).lower()
                if proceed_all != "yes":
                    print("Operation cancelled by user.")
                    return
        except KeyboardInterrupt:
            print("\nOperation cancelled by user (Ctrl+C).")
            return

    success_count = 0
    failure_count = 0

    for wsl_path_str in resolved_wsl_paths:
        # The create_windows_symlink function uses its confirm_deletion and dry_run params
        result = create_windows_symlink(
            wsl_item_path_str=wsl_path_str,
            wsl_config_root_str=resolved_wsl_config_root,
            wsl_distro_name=args.distro_name,
            confirm_deletion=effective_confirm_deletion,  # Pass the mode-adjusted flag
            dry_run=args.dry_run,
        )
        # create_windows_symlink returns True for success OR user skip, False for error
        # We need to distinguish true success from user skip if we want to count them differently.
        # For now, the function's print output makes this clear. Let's assume True means "no error occurred".
        # A more robust way would be for the function to return an enum/status code.
        # Given the current return: True if not error, False if error.
        # If user skips, it returns True. We need to check input() result somehow if we want specific count.
        # The function currently prints "User skipped deletion... Link creation will be skipped".
        # Let's refine this: if user skips, we consider it a "skip", not a "success".
        # This requires create_windows_symlink to return a more detailed status.
        # For simplicity of this iteration: let's assume the printout is enough.

        if result:  # True means no script error (could be success or user skip)
            # To differentiate, we'd need create_windows_symlink to return specific codes
            # For now, if it returns True, we call it a success in terms of script execution.
            # The print statements inside create_windows_symlink clarify user skips.
            success_count += 1
        else:  # False means script error
            failure_count += 1

        # A small pause between items if not in CI mode
        if not args.ci_mode and not args.dry_run and len(resolved_wsl_paths) > 1:
            time.sleep(0.1)

    print("\n--- Summary ---")
    print(f"Items attempted: {len(resolved_wsl_paths)}")
    print(
        f"Successfully processed (or skipped by user choice without error): {success_count} item(s)"
    )
    print(f"Failed to process due to errors: {failure_count} item(s)")
    if args.dry_run:
        print("NOTE: This was a DRY RUN. No actual changes were made.")
    if failure_count > 0:
        print("Please review errors above.")
        exit(1)  # Exit with error code if there were failures


if __name__ == "__main__":
    # Quick User Guidance:
    # 1. Windows to WSL stotage
    # 1.1 put your files manually
    # put your Windows config file in ~/dotfiles/c.mnt/<your_windows_path>
    # e.g. C:\Users\zion\test.conf ==> ~/dotfiles/c.mnt/Users/zion/test.conf
    # 1.2 run this script to create a symlink in WSL (files recommended, directories may cause issues, i.e., it may include extra files)
    # <TODO: script to auto move files from Windows to WSL storage>
    # 2. WSL to Windows storage
    # 2.1 quick proceed all files, just run:
    #  $ fd . c.mnt -H -t f -x python scripts.meta/config_linker.py --no-confirm-deletion {}
    # 2.2 specify files or directories manually:
    #  $ python scripts.meta/config_linker.py ~/dotfiles/c.mnt/Users/zion/.config <other_files_or_dirs>
    main_cli()
