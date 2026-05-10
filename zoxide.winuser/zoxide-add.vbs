' zoxide-add.vbs - 无窗口添加目录到zoxide
' 用法：wscript.exe zoxide-add.vbs "要添加的目录路径"

On Error Resume Next
Dim WshShell, strPath
Set WshShell = CreateObject("WScript.Shell")

' 获取路径并添加双引号
strPath = Chr(34) & WScript.Arguments(0) & Chr(34)

' 执行zoxide add，0表示隐藏窗口，False表示异步执行
WshShell.Run "zoxide add " & strPath, 0, False

' 清理
Set WshShell = Nothing
