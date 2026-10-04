#Requires AutoHotkey v2.0
#SingleInstance Force
; ============================================================================
; 微信 4.x 自动登录并关闭主窗口
; 流程：启动微信 -> 聚焦登录窗口 -> 发送回车 -> 检测窗口变大(登录成功) -> 关闭主窗口
;
; 说明：微信 4.x 的登录窗口与主窗口标题相同（均为"微信"），无法用标题区分，
;       只能通过窗口大小判断。本脚本把窗口实际像素按窗口所在显示器的 DPI
;       换算为"逻辑像素"（96 DPI 基准）再比较，自动适配 100%/125%/150%/200%
;       以及多显示器不同缩放的场景。
; ============================================================================

; ---------- 可调参数 ----------
EXE_NAME        := "Weixin.exe"     ; 微信 4.x 进程名
WIN_MATCH       := "微信 ahk_exe " EXE_NAME
MAIN_MIN_W      := 500              ; 主界面最小宽度（逻辑像素，96 DPI 基准）
MAIN_MIN_H      := 560              ; 主界面最小高度（逻辑像素）
GROW_RATIO      := 1.3              ; 相对登录窗口尺寸放大的判定比例
AUTH_TIMEOUT_S  := 300              ; 等待登录完成的最长时间（秒），超时静默退出
POLL_MS         := 250              ; 轮询间隔（毫秒）
CLOSE_DELAY_MS  := 800              ; 检测到登录成功后延迟再关闭（毫秒）

SetTitleMatchMode(2)                ; 标题"包含"匹配

; ---------- 1. 启动微信 ----------
if !WinExist("ahk_exe " EXE_NAME) {
    Run(FindWeChat())
}

; ---------- 2. 等待微信窗口出现 ----------
try {
    WinWait(WIN_MATCH, , 60)
} catch {
    MsgBox("等待微信窗口超时，脚本退出")
    ExitApp()
}

; 若启动后直接就是主界面（已勾选自动登录、秒进），直接关闭
if (id := FindMainWindow()) {
    Sleep(CLOSE_DELAY_MS)
    CloseMain(id)
    ExitApp()
}

; ---------- 3. 记录登录窗口逻辑尺寸，聚焦并发送回车 ----------
hwnd := WinGetID(WIN_MATCH)
GetLogicalSize(hwnd, &loginW, &loginH)

WinActivate(hwnd)
try WinWaitActive(hwnd, , 5)
Sleep(200)
Send("{Enter}")

; ---------- 4. 轮询窗口尺寸：明显变大 = 登录成功 ----------
startTick := A_TickCount
lastEnter := startTick
while (A_TickCount - startTick < AUTH_TIMEOUT_S * 1000) {
    if (id := FindMainWindow(loginW, loginH)) {
        Sleep(CLOSE_DELAY_MS)
        CloseMain(id)
        ExitApp()
    }
    ; 前 15 秒内每 3 秒补发一次回车，防止焦点被抢导致首次回车无效
    if (A_TickCount - startTick < 15000 && A_TickCount - lastEnter > 3000 && WinExist(hwnd)) {
        WinActivate(hwnd)
        Send("{Enter}")
        lastEnter := A_TickCount
    }
    Sleep(POLL_MS)
}
ExitApp()   ; 超时静默退出，不干扰手动操作

; ============================================================================
; 函数
; ============================================================================

; 枚举所有微信顶层窗口，找到"主界面尺寸"的窗口则返回其 HWND，否则返回 0
FindMainWindow(refW := 0, refH := 0) {
    global WIN_MATCH, MAIN_MIN_W, MAIN_MIN_H, GROW_RATIO
    for id in WinGetList(WIN_MATCH) {
        GetLogicalSize(id, &w, &h)
        ; 判定一：绝对尺寸达到主界面下限（登录窗口逻辑尺寸约 280x450，远低于此）
        if (w >= MAIN_MIN_W || h >= MAIN_MIN_H)
            return id
        ; 判定二：相对启动时记录的登录窗口尺寸明显变大
        if (refW && refH && (w >= refW * GROW_RATIO || h >= refH * GROW_RATIO))
            return id
    }
    return 0
}

; 读取窗口尺寸并换算为 96 DPI 下的逻辑像素，适配高 DPI / 多显示器
GetLogicalSize(hwnd, &w, &h) {
    WinGetPos(, , &pw, &ph, hwnd)
    dpi := 96
    ; GetDpiForWindow 需要 Win10 1607+，失败则按 96 DPI 处理
    try dpi := DllCall("user32\GetDpiForWindow", "ptr", hwnd, "uint")
    if (dpi <= 0)
        dpi := 96
    w := pw * 96 / dpi
    h := ph * 96 / dpi
}

; 关闭微信主窗口
; 注意：微信默认行为是关闭窗口后最小化到系统托盘，进程不退出；
;       如需彻底退出微信，把 WinClose 一行替换为：ProcessClose(EXE_NAME)
CloseMain(hwnd) {
    WinClose(hwnd)
}

; 定位微信 4.x 安装路径
FindWeChat() {
    global EXE_NAME
    candidates := [
        A_ProgramFiles "\Tencent\Weixin\Weixin.exe",
        EnvGet("ProgramFiles(x86)") "\Tencent\Weixin\Weixin.exe",
        EnvGet("LocalAppData") "\Tencent\Weixin\Weixin.exe"
    ]
    for p in candidates
        if FileExist(p)
            return p
    ; 注册表兜底
    for key in ["HKCU\Software\Tencent\Weixin",
                "HKLM\SOFTWARE\Tencent\Weixin",
                "HKLM\SOFTWARE\WOW6432Node\Tencent\Weixin"] {
        try {
            dir := RegRead(key, "InstallPath")
            if FileExist(dir "\Weixin.exe")
                return dir "\Weixin.exe"
        }
    }
    return EXE_NAME   ; 最终兜底：依赖系统 PATH / App Paths
}
