#Requires AutoHotkey v2.0.2
#SingleInstance Force

; 定义一个函数来执行komorebi命令
Komorebic(cmd) {
    RunWait(format("komorebic.exe {}", cmd), , "Hide")
}


; 焦点窗口
<!h::Komorebic("focus left")  ; Alt+H 焦点左移
<!j::Komorebic("focus down")  ; Alt+J 焦点下移
<!k::Komorebic("focus up")  ; Alt+K 焦点上移
<!l::Komorebic("focus right")  ; Alt+L 焦点右移

; 移动窗口
<!+h::Komorebic("move left")  ; Alt+Shift+H 窗口左移
<!+j::Komorebic("move down")  ; Alt+Shift+J 窗口下移
<!+k::Komorebic("move up")  ; Alt+Shift+K 窗口上移
<!+l::Komorebic("move right")  ; Alt+Shift+L 窗口右移

; 工作区切换
<!1::Komorebic("focus-workspace 0")
<!2::Komorebic("focus-workspace 1")
<!3::Komorebic("focus-workspace 2")
<!4::Komorebic("focus-workspace 3")
<!5::Komorebic("focus-workspace 4")

; 绑定更多komorebi快捷键
<!+1::Komorebic("move-to-workspace 0")
<!+2::Komorebic("move-to-workspace 1")
<!+3::Komorebic("move-to-workspace 2")
<!+4::Komorebic("move-to-workspace 3")
<!+5::Komorebic("move-to-workspace 4")

; 调整窗口大小
<!=::Komorebic("resize-axis horizontal increase")  ; Alt+= 横向增加大小
<!-::Komorebic("resize-axis horizontal decrease")  ; Alt+- 横向减少大小
<!+=::Komorebic("resize-axis vertical increase")  ; Alt+Shift+= 纵向增加大小
<!+_::Komorebic("resize-axis vertical decrease")  ; Alt+Shift+_ 纵向减少大小

; 窗口管理选项
ReloadKomorebi() {
  RunWait("taskkill /F /IM komorebi.exe")
  RunWait("komorebic-no-console.exe start --ahk")
}
<!+r::ReloadKomorebi()
<!p::Komorebic("toggle-pause")  ; Alt+P 暂停/恢复窗口管理

<!+Enter::Komorebic("promote")
<!+Esc::Komorebic("close")
<!m::Komorebic("minimize")
<!f::Komorebic("toggle-maximize")
<!t::Komorebic("toggle-float")

<![::Komorebic("cycle-focus previous")
<!]::Komorebic("cycle-focus next")

<!+[::Komorebic("cycle-move previous")
<!+]::Komorebic("cycle-move next")

; Apps
<!Enter::Run("wt.exe")
<!Esc::Run(format('wt.exe -p Arch -d "{}"', FileRead("C:\Users\zion\.workdir")))
