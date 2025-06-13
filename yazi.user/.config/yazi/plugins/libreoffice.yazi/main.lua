local M = {}

local function cmd_exists(cmd)
    local ok, res = pcall(function()
        return io.popen(cmd):read("*a")
    end)

    return ok and res ~= ""
end

local function cmd_path()
    local cmd = "libreoffice"
    local os_family = ya.target_family()
    local os_name = ya.target_os()
    local prefix = (os_family == "windows" and "where" or "command -v") .. " "

    if cmd_exists(prefix .. cmd) then return cmd end

    if os_name == "macos" then
        local cmd = "/Applications/LibreOffice.app/Contents/MacOS/soffice"
        if cmd_exists(prefix .. cmd) then return cmd end
    elseif os_name == "linux" then
        local cmd = "/usr/bin/libreoffice"
        if cmd_exists(prefix .. cmd) then return cmd end
    elseif os_name == "windows" then
        local cmd = {
            "C:\\Program Files\\LibreOffice\\program\\soffice.exe",
            os.getenv("USERPROFILE") .. "\\scoop\\apps\\libreoffice\\current\\program\\soffice.exe",
        }

        for _, c in ipairs(cmd) do
            if cmd_exists(prefix .. c) then
                return c
            end
        end
    end

    return ""
end

local function base_dir(path)
    return string.match(path, "^(.*[/\\])")
end

local function file_basename(path)
    return string.match(path, "([^/\\]+)%.")
end

function M:peek(job)
    if cmd_path() == "" then
        return
    end

    local start, cache = os.clock(), ya.file_cache(job)
    if not cache then
        return
    end

    local ok, err = self:preload(job)
    if not ok or err then
        return
    end

    ya.sleep(math.max(0, rt.preview.image_delay / 1000 + start - os.clock()))

    local _, err = ya.image_show(cache, job.area)
    ya.preview_widget(job, err and ui.Text(tostring(err)):area(job.area):wrap(ui.Text.WRAP))
end

function M:seek(job)
    local h = cx.active.current.hovered
    if h and h.url == job.file.url then
        local step = ya.clamp(-1, job.units, 1)
        ya.emit("peek", { math.max(0, cx.active.preview.skip + step), only_if = job.file.url })
    end
end

function M:preload(job)
    local cmd = cmd_path()
    local cache = ya.file_cache(job)
    if not cache or fs.cha(cache) or cmd == "" then
        return true
    end
    local fbname = file_basename(tostring(job.file.url))
    local outdir = base_dir(tostring(cache))

    -- stylua: ignore
    local output, err = Command(cmd)
        :arg {
            "--headless",
            "--convert-to",
            "jpg",
            tostring(job.file.url),
            '--outdir',
            outdir
        }
        :stdout(Command.PIPED)
        :stderr(Command.PIPED)
        :output()

    if not output then
        return true, Err("Failed to start `libreoffice`, error: %s", err)
    elseif not output.status.success then
        return true, Err("Failed to convert PDF to image, stderr: %s", output.stderr)
    end

    local ok, err = os.rename(outdir .. fbname .. '.jpg', tostring(cache))

    if ok then
        return true
    else
        return false, Err("Failed to rename `%s.jpg` to `%s`, error: %s", cache, cache, err)
    end
end

return M
