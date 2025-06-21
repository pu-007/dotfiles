local M = {}

function M:setup(cfg)
	self.cfg = cfg
end

---
-- 解析路径为最终的、真实的绝对路径。
-- 此函数假定脚本的当前工作目录 (CWD) 已经是正确的上下文。
-- 它能处理任意路径（相对/绝对/含符号链接）。
--
-- @param path_str string        需要解析的路径。
-- @return string|nil, string   成功则返回真实的绝对路径；失败则返回 nil 和错误信息。
--
local function to_abs_path(path_str)
	-- 使用 %q 为 shell 命令安全地引用路径。
	local safe_path = string.format("%q", path_str)

	-- 命令现在非常简单，readlink 会自动使用当前进程的 CWD 来解析相对路径。
	local command = "readlink -f " .. safe_path

	local f = io.popen(command)
	if not f then
		return nil, "Failed to execute 'readlink' command."
	end

	local real_path = f:read("*a")
	local success, _, exit_code = f:close()

	-- 如果命令执行失败或没有返回任何内容，则解析失败。
	if not success or exit_code ~= 0 or real_path == "" then
		return nil, "Failed to resolve path: " .. path_str
	end

	-- 清理并返回结果。
	return real_path:gsub("[\r\n]$", "")
end

-- Converts a WSL path to a Windows path.
-- @param wsl_path (string) The WSL path to convert.
-- @param distro (string) The name of the WSL distribution.
-- @return (string) The converted Windows path.
local function to_win_path(wsl_path, distro)
	local win_path
	-- Check if it's a mounted Windows path (e.g., /mnt/c/...)
	if string.sub(wsl_path, 1, 5) == "/mnt/" then
		local drive = string.sub(wsl_path, 6, 6)
		win_path = string.upper(drive) .. ":" .. string.sub(wsl_path, 7)
		win_path = string.gsub(win_path, "/", "\\")
	else
		-- Assume it's a WSL-native path (e.g., /home/user/...)
		win_path = "\\\\wsl.localhost\\" .. distro .. wsl_path
		win_path = string.gsub(win_path, "/", "\\")
	end
	return win_path
end

local get_current_abs_path = ya.sync(function()
	local current_file = cx.active.current.hovered
	if current_file.cha.is_link then
		return to_abs_path(tostring(current_file.link_to))
	else
		-- to deal with a file that is not a link but a parent directory
		return to_abs_path(tostring(current_file.url))
	end
end)

-- it's necessary to use ya.sync to get sync context vars
-- bacause the plugin is async for default
local get_cfg = ya.sync(function(_)
	return M.cfg
end)

function M.entry()
	local cfg = get_cfg()
	local distro = cfg.wsl_distro or "Arch"
	local quicklook_exe_wsl = cfg.quicklook_path or "/mnt/c/Users/zion/AppData/Local/Programs/QuickLook/QuickLook.exe"
	local file_path_wsl = get_current_abs_path()

	local file_path_win = "'" .. to_win_path(file_path_wsl, distro) .. "'"

	if cfg.debug then
		ya.dbg("QuickLook Plugin:")
		ya.dbg("==>WSL Path: " .. file_path_wsl)
		ya.dbg("==>Windows Path: " .. file_path_win)
		ya.dbg("==>QuickLook WSL Path: " .. quicklook_exe_wsl)
	end

	os.execute(quicklook_exe_wsl .. " " .. file_path_win .. " -top")

	local pipe = io.popen("pwsh.exe -Command python.exe -", "w")

	pipe:write([[
from time import time, sleep
from pyautogui import getWindowsWithTitle

end_time = time() + 3
while time() <= end_time:
    windows = getWindowsWithTitle("QuickLook")
    if windows:
        for window in windows:
            window.activate()
        break
]])
	pipe:close()
end

return M
