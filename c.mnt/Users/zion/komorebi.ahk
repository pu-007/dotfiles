#Requires AutoHotkey v2.0.2
#SingleInstance Force

; ### Alt 键交换 (持久化版本) ###

; 保存配置文件路径（在脚本同目录）
iniFile := A_ScriptDir "\komorebiahk_settings.ini"

; 启动时读取 altSwapped 值（默认 false）
global altSwapped := (IniRead(iniFile, "Settings", "AltSwapped", "0") = "1")

Esc & F1:: {
    global altSwapped, iniFile
    altSwapped := !altSwapped
    IniWrite(altSwapped ? "1" : "0", iniFile, "Settings", "AltSwapped")
    ToolTip "Alt 交换: " (altSwapped ? "开启" : "关闭")
    SetTimer () => ToolTip(), -1000
}

#HotIf altSwapped
~LAlt::RAlt ; 物理左 Alt -> lalt + ralt (间接让 komorebi keybindings 失效)
$RAlt::LAlt ; 物理右 Alt -> lalt
#HotIf

; ### Komorebi ###

; 定义一个函数来执行komorebi命令
Komorebic(cmd) {
    RunWait(format("komorebic.exe {}", cmd), , "Hide")
}


; 焦点窗口
<!h:: Komorebic("focus left")  ; Alt+H 焦点左移
<!j:: Komorebic("focus down")  ; Alt+J 焦点下移
<!k:: Komorebic("focus up")  ; Alt+K 焦点上移
<!l:: Komorebic("focus right")  ; Alt+L 焦点右移

; 移动窗口
<!+h:: Komorebic("move left")  ; Alt+Shift+H 窗口左移
<!+j:: Komorebic("move down")  ; Alt+Shift+J 窗口下移
<!+k:: Komorebic("move up")  ; Alt+Shift+K 窗口上移
<!+l:: Komorebic("move right")  ; Alt+Shift+L 窗口右移

; 工作区切换
<!1:: Komorebic("focus-workspace 0")
<!2:: Komorebic("focus-workspace 1")
<!3:: Komorebic("focus-workspace 2")
<!4:: Komorebic("focus-workspace 3")
<!5:: Komorebic("focus-workspace 4")
<!6:: Komorebic("focus-workspace 5")
<!7:: Komorebic("focus-workspace 6")
<!8:: Komorebic("focus-workspace 7")
<!9:: Komorebic("focus-workspace 8")
<!0:: Komorebic("focus-workspace 9")

; 绑定更多komorebi快捷键
<!+1:: Komorebic("move-to-workspace 0")
<!+2:: Komorebic("move-to-workspace 1")
<!+3:: Komorebic("move-to-workspace 2")
<!+4:: Komorebic("move-to-workspace 3")
<!+5:: Komorebic("move-to-workspace 4")
<!+6:: Komorebic("move-to-workspace 5")
<!+7:: Komorebic("move-to-workspace 6")
<!+8:: Komorebic("move-to-workspace 7")
<!+9:: Komorebic("move-to-workspace 8")
<!+0:: Komorebic("move-to-workspace 9")

; 调整窗口大小
<!=:: Komorebic("resize-axis horizontal increase")  ; Alt+= 横向增加大小
<!-:: Komorebic("resize-axis horizontal decrease")  ; Alt+- 横向减少大小
<!+=:: Komorebic("resize-axis vertical increase")  ; Alt+Shift+= 纵向增加大小
<!+_:: Komorebic("resize-axis vertical decrease")  ; Alt+Shift+_ 纵向减少大小
; Resize the focused window
<!+left:: Komorebic("resize left decrease")
<!+right:: Komorebic("resize left increase")
<!+up:: Komorebic("resize up decrease")
<!+down:: Komorebic("resize up increase")

; 窗口管理选项
; alt+r 重新加载 komorebi
<!r:: {
    RunWait("taskkill /F /IM komorebi.exe", , "Hide")
    Run("komorebic-no-console.exe start --ahk")
}
; alt+shift+r 重新加载 komorebi, yasb
<!+r:: {
    RunWait("taskkill /F /IM komorebi.exe", , "Hide")
    RunWait("taskkill /F /IM yasb.exe", , "Hide")
    Run("komorebic-no-console.exe start --ahk")
    Run("yasb.exe")
}
; ctrl+shift+r 重新加载 komorebi, yasb, explorer
^+r:: {
    RunWait("taskkill /F /IM komorebi.exe", , "Hide")
    RunWait("taskkill /F /IM yasb.exe", , "Hide")
    RunWait("taskkill /F /IM explorer.exe", , "Hide")
    Run("explorer.exe")
    Run("komorebic-no-console.exe start --ahk")
    Run("yasb.exe")
}

<!+p:: Komorebic("toggle-pause")  ; Alt+P 暂停/恢复窗口管理

<!p:: Komorebic("promote")
<!m:: Komorebic("toggle-maximize")
<!+m:: Komorebic("minimize")
<!z:: {
  try {
    WinClose("A")
  } catch Error as e {
    MsgBox "Failed to close window: " e.Message
  }
}
<!+z::
{
    try {
        pid := WinGetPID("A")  ; Get process ID of active windo2
        ProcessClose(pid)      ; Force kill process
    } catch Error as e {
        MsgBox "Failed to kill process: " e.Message
    }
}
; Toggle monocle layout (full screen for focused window while preserving tiling)
<!f:: Komorebic("toggle-monocle")
<!t:: Komorebic("toggle-float")

; Cycle Focus (previous/next)
<!$[:: Komorebic("cycle-focus previous")
<!$]:: Komorebic("cycle-focus next")

; Cycle Move (previous/next)
<!${:: Komorebic("cycle-move previous")
<!$}:: Komorebic("cycle-move next")

; Cycle Layout (previous/next)
<!PgUp:: Komorebic("cycle-layout previous")
<!PgDn:: Komorebic("cycle-layout next")

; Cycle through stacked windows
<!;:: Komorebic("cycle-stack previous")
<!':: Komorebic("cycle-stack next")

; Layout Shortcuts
<!,:: Komorebic("change-layout vertical-stack")
<!<:: Komorebic("change-layout right-main-vertical-stack")
<!.:: Komorebic("change-layout bsp")
<!>:: Komorebic("change-layout grid")
<!/:: Komorebic("change-layout ultrawide-vertical-stack")
<!?:: Komorebic("change-layout horizontal-stack")

; Flip the current layout
<!+x:: Komorebic("flip-layout x")
<!+y:: Komorebic("flip-layout y")

; Apps
; 启动终端
<!Enter:: Run("wt.exe")
<!+Enter:: Run(format('wt.exe -p Arch -d "{}"', FileRead("C:\Users\zion\.workdir")))

; Focus monitors
<!F1:: Komorebic("focus-monitor 0")
<!F2:: Komorebic("focus-monitor 1")

; Cycle through monitors
<!+F1:: Komorebic("cycle-monitor previous")
<!+F2:: Komorebic("cycle-monitor next")

; Stack windows in a direction
<!\:: Komorebic("stack left")
<!+\:: Komorebic("unstack")
; ### hyper keys and terminal app ###
; some are defiend in uTools, ecopaste
; Hyper + W 启动/切换微信
^+!#w:: {
    ; 检查 Weixin.exe 进程是否存在
    if !ProcessExist("Weixin.exe") {
        ; 如果不存在，就运行它
        Run("C:\Program Files\Tencent\Weixin\Weixin.exe")
    } else {
        ; Send ctrl+shift+alt+F12 to toggle WeChat
        Send("^+!{F12}")
    }
}
; Hyper + Q 启动/切换 QQ
^+!#q:: {
    ; 检查 Weixin.exe 进程是否存在
    if !ProcessExist("QQ.exe") {
        ; 如果不存在，就运行它
        Run("C:\Program Files\Tencent\QQNT\QQ.exe")
    } else {
        ; Send win+ctrl+F12 to toggle QQ
        Send("^#{F12}")
    }
}
; Hyper + n nvim edit
^+!#n:: Run("wt.exe -p Arch wsl nvim -c 'read !win32yank.exe -o'")
#y:: Run("wsl.exe zsh -ic 'y /mnt/d/Downloads/'")
