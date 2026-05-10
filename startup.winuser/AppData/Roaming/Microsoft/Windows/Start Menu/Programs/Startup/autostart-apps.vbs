' 检查是否带参数运行，如果不带，说明是第一次启动，需要申请提权
If WScript.Arguments.Length = 0 Then
    Set objShell = CreateObject("Shell.Application")
    ' 使用 runas 动作重新以管理员权限启动自身，并传入一个参数防止死循环
    objShell.ShellExecute "wscript.exe", """" & WScript.ScriptFullName & """ Admin", "", "runas", 1
    WScript.Quit
End If

' 下面的代码将在管理员权限下执行
Set ws = WScript.CreateObject("WScript.Shell")

' 以管理员权限静默执行你的命令
ws.run "pixi run -m C:\Users\zionpu\autostart\pixi.toml autostart", 0

Set ws = Nothing
