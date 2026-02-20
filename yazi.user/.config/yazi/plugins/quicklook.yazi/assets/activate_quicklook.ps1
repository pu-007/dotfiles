Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32 {
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern int GetWindowText(IntPtr hWnd, System.Text.StringBuilder text, int count);
    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
}
"@
$sw = [Diagnostics.Stopwatch]::StartNew()
while ($sw.ElapsedMilliseconds -lt 3000) {
    $found = $false
    [Win32]::EnumWindows({
        param($hWnd, $lParam)
        $sb = New-Object System.Text.StringBuilder 256
        [Win32]::GetWindowText($hWnd, $sb, $sb.Capacity) | Out-Null
        if ($sb.ToString().StartsWith("QuickLook - ")) {
            [Win32]::SetForegroundWindow($hWnd) | Out-Null
            $script:found = $true
            return $false # 停止遍历
        }
        return $true # 继续遍历
    }, [IntPtr]::Zero) | Out-Null

    if ($found) { break }
    Start-Sleep -Milliseconds 50
}
