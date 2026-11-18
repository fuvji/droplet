param(
    [string]$OutDir = "site/public/assets",
    [string]$DpiOutDir = "site/public/assets/dpi"
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'target\x86_64-pc-windows-gnu\release\droplet.exe'
if (-not (Test-Path -LiteralPath $exe)) {
    throw 'Build the release executable before capturing screenshots.'
}

$out = Join-Path $root $OutDir
$dpiOut = Join-Path $root $DpiOutDir
New-Item -ItemType Directory -Force -Path $out | Out-Null

# Capture from a fresh, isolated profile. Using the caller's real config makes
# the published images depend on whatever bindings and toggles happen to be on
# this machine (and can preserve a legacy duplicate binding). The seeded
# profile makes every capture deterministic and demonstrates Anchor ownership
# of XButton1; normalize_bindings clears the old Breach owner on load.
$captureRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("droplet-capture-" + [Guid]::NewGuid().ToString("N"))
$captureData = Join-Path $captureRoot "Droplet"
New-Item -ItemType Directory -Force -Path $captureData | Out-Null
Set-Content -LiteralPath (Join-Path $captureData "config.ini") -Encoding ASCII -Value @"
[T]
v0=1
vP=1
v6Bn=XButton1
"@
$savedEnv = @{
    LOCALAPPDATA = $env:LOCALAPPDATA
    TEMP = $env:TEMP
    TMP = $env:TMP
}

Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @"
using System;
using System.Drawing;
using System.Runtime.InteropServices;
using System.Text;

public static class DropletTabs {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string className, string windowName);

    public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int max);

    public static IntPtr FindWindowForProcess(uint targetPid) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((hwnd, lParam) => {
            uint pid;
            GetWindowThreadProcessId(hwnd, out pid);
            if (pid == targetPid) {
                var title = new StringBuilder(128);
                GetWindowText(hwnd, title, title.Capacity);
                if (title.ToString() == "Droplet") {
                    found = hwnd;
                    return false;
                }
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hwnd, int command);

    [DllImport("user32.dll")]
    public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int cx, int cy, uint flags);

    [DllImport("user32.dll")]
    public static extern IntPtr SendMessage(IntPtr hwnd, uint msg, IntPtr wParam, ref RECT lParam);

    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);

    [DllImport("user32.dll")]
    public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

    [DllImport("gdi32.dll")]
    public static extern IntPtr CreateRectRgn(int left, int top, int right, int bottom);

    [DllImport("user32.dll")]
    public static extern int GetWindowRgn(IntPtr hwnd, IntPtr region);

    [DllImport("gdi32.dll")]
    public static extern bool PtInRegion(IntPtr region, int x, int y);

    [DllImport("gdi32.dll")]
    public static extern bool DeleteObject(IntPtr obj);

    // Apply the live window's own region, not a hardcoded radius. This makes
    // the published PNG prove the real silhouette and cannot hide a square
    // frame with a decorative mask.
    public static void ApplyWindowRegionAlpha(Bitmap bitmap, IntPtr hwnd, int radius) {
        IntPtr region = CreateRectRgn(0, 0, 0, 0);
        try {
            if (GetWindowRgn(hwnd, region) == 0) {
                // The layered-alpha window has no binary region. Apply the same
                // logical silhouette the app rendered instead of leaving a
                // square screenshot. The radius is physical, so it tracks the
                // current DPI scale.
                ApplyRoundedAlpha(bitmap, radius);
                return;
            }
            for (int y = 0; y < bitmap.Height; y++) {
                for (int x = 0; x < bitmap.Width; x++) {
                    if (!PtInRegion(region, x, y)) {
                        bitmap.SetPixel(x, y, Color.Transparent);
                    }
                }
            }
        } finally {
            DeleteObject(region);
        }
    }

    public static void ApplyRoundedAlpha(Bitmap bitmap, int radius) {
        int w = bitmap.Width;
        int h = bitmap.Height;
        const int Samples = 4;
        for (int y = 0; y < radius; y++) {
            for (int x = 0; x < radius; x++) {
                foreach (bool right in new bool[] { false, true }) {
                    foreach (bool bottom in new bool[] { false, true }) {
                        int px = right ? w - 1 - x : x;
                        int py = bottom ? h - 1 - y : y;
                        int inside = 0;
                        for (int sy = 0; sy < Samples; sy++) {
                            for (int sx = 0; sx < Samples; sx++) {
                                double fx = x + (sx + 0.5) / Samples;
                                double fy = y + (sy + 0.5) / Samples;
                                double dx = fx - radius;
                                double dy = fy - radius;
                                if (dx * dx + dy * dy <= radius * radius) inside++;
                            }
                        }
                        double coverage = inside / (double)(Samples * Samples);
                        Color c = bitmap.GetPixel(px, py);
                        bitmap.SetPixel(px, py, Color.FromArgb((int)(255 * coverage), c.R, c.G, c.B));
                    }
                }
            }
        }
    }
}
"@

function Save-Panel(
    [IntPtr]$Hwnd,
    [string]$Name,
    [string]$TargetDir,
    [int]$ExpectedWidth,
    [int]$ExpectedHeight,
    [int]$Radius
) {
    $rect = New-Object DropletTabs+RECT
    if (-not [DropletTabs]::GetWindowRect($Hwnd, [ref]$rect)) {
        throw 'GetWindowRect failed.'
    }
    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -ne $ExpectedWidth -or $height -ne $ExpectedHeight) {
        throw "Expected ${ExpectedWidth}x${ExpectedHeight}, got ${width}x${height}."
    }

    $bitmap = New-Object System.Drawing.Bitmap($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $hdc = $graphics.GetHdc()
        try {
            # PrintWindow captures the window rather than the desktop behind its
            # acrylic. CopyFromScreen would include unrelated windows that show
            # through the translucent panel.
            if (-not [DropletTabs]::PrintWindow($Hwnd, $hdc, 2)) {
                throw 'PrintWindow failed.'
            }
        } finally {
            $graphics.ReleaseHdc($hdc)
        }
        # Use the live region. The old hardcoded GraphicsPath could make a
        # square DWM frame look rounded in the screenshot even when the app
        # itself did not clip that frame.
        [DropletTabs]::ApplyWindowRegionAlpha($bitmap, $Hwnd, $Radius)
        $target = Join-Path $TargetDir $Name
        $bitmap.Save($target, [System.Drawing.Imaging.ImageFormat]::Png)
        $image = [System.Drawing.Image]::FromFile($target)
        try {
            if ($image.Width -ne $ExpectedWidth -or $image.Height -ne $ExpectedHeight) {
                throw "Saved capture is $($image.Width)x$($image.Height), expected ${ExpectedWidth}x${ExpectedHeight}."
            }
        } finally {
            $image.Dispose()
        }
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function Click-Tab([IntPtr]$Hwnd, [int]$X, [int]$Y) {
    $lParam = [IntPtr]::new([int](($Y -shl 16) -bor ($X -band 0xFFFF)))
    [DropletTabs]::PostMessage($Hwnd, 0x0201, [IntPtr]::new(1), $lParam) | Out-Null
    Start-Sleep -Milliseconds 100
    [DropletTabs]::PostMessage($Hwnd, 0x0202, [IntPtr]::Zero, $lParam) | Out-Null
    Start-Sleep -Milliseconds 350
}

function Set-PanelDpi([IntPtr]$Hwnd, [int]$Percent) {
    $scale = $Percent / 100.0
    $expectedWidth = [int][Math]::Round(624 * $scale)
    $expectedHeight = [int][Math]::Round(408 * $scale)
    $dpi = [int][Math]::Round(96 * $scale)
    $current = New-Object DropletTabs+RECT
    if (-not [DropletTabs]::GetWindowRect($Hwnd, [ref]$current)) {
        throw 'GetWindowRect failed before the DPI change.'
    }
    $suggested = New-Object DropletTabs+RECT
    $suggested.Left = $current.Left
    $suggested.Top = $current.Top
    $suggested.Right = $current.Left + $expectedWidth
    $suggested.Bottom = $current.Top + $expectedHeight
    $dpiWord = $dpi -bor ($dpi -shl 16)
    [DropletTabs]::SendMessage($Hwnd, 0x02E0, [IntPtr]::new($dpiWord), [ref]$suggested) | Out-Null
    Start-Sleep -Milliseconds 600
    $actual = New-Object DropletTabs+RECT
    if (-not [DropletTabs]::GetWindowRect($Hwnd, [ref]$actual)) {
        throw 'GetWindowRect failed after the DPI change.'
    }
    $actualWidth = $actual.Right - $actual.Left
    $actualHeight = $actual.Bottom - $actual.Top
    if ($actualWidth -ne $expectedWidth -or $actualHeight -ne $expectedHeight) {
        throw "DPI ${Percent}% expected ${expectedWidth}x${expectedHeight}, got ${actualWidth}x${actualHeight}."
    }
    Write-Output "DPI ${Percent}% -> ${expectedWidth}x${expectedHeight}"
}

# The PowerShell host may be DPI-unaware. Make it per-monitor v2 before it
# reads any window rect or screen pixels; otherwise Windows virtualizes a
# 120-DPI panel back to 624x408 and clips the capture.
[DropletTabs]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null

$process = $null
try {
    $env:LOCALAPPDATA = $captureRoot
    $env:TEMP = $captureRoot
    $env:TMP = $captureRoot
    $process = Start-Process -FilePath $exe -WorkingDirectory $root -PassThru
    Start-Sleep -Seconds 2
    # Target this process, not the first Droplet window on the desktop. A
    # user-launched instance may be sitting at another size; FindWindow would
    # silently capture that instance instead of the release build under test.
    $hwnd = [DropletTabs]::FindWindowForProcess([uint32]$process.Id)
    if ($hwnd -eq [IntPtr]::Zero) {
        throw 'The Droplet panel window was not created for this process.'
    }
    $targetPid = [uint32]0
    [DropletTabs]::GetWindowThreadProcessId($hwnd, [ref]$targetPid) | Out-Null
    Write-Output "Capture window hwnd=$hwnd pid=$targetPid expected=$($process.Id)"

    [DropletTabs]::ShowWindow($hwnd, 5) | Out-Null
    Start-Sleep -Milliseconds 700

    # Force the canonical 624x408 assets to 100%. This exercises the same
    # WM_DPICHANGED path used by a real monitor move on a 96 DPI host.
    Set-PanelDpi $hwnd 100
    New-Item -ItemType Directory -Force -Path $dpiOut | Out-Null

    # Rail items start at y=52 with a 34px height and 36px pitch at 96 DPI.
    # These are physical client coordinates, matching App::on_pointer.
    $tabs = @(
        @{ Name = 'droplet-core.png';    Y = 69;  X = 50 },
        @{ Name = 'droplet-crystal.png'; Y = 105; X = 50 },
        @{ Name = 'droplet-assist.png';  Y = 141; X = 50 },
        @{ Name = 'droplet-system.png';  Y = 177; X = 50 }
    )

    # Live DPI evidence first. The Core rail target scales with the panel; the
    # capture stays at physical pixels so the four corner alphas can be
    # checked without a resampling step. The canonical four are recaptured
    # after this pass, from the same settled window, so a stale first paint
    # cannot leak into the published images.
    foreach ($percent in @(100, 125, 150)) {
        Set-PanelDpi $hwnd $percent
        $scale = $percent / 100.0
        Click-Tab $hwnd ([int](50 * $scale)) ([int](69 * $scale))
        $width = [int][Math]::Round(624 * $scale)
        $height = [int][Math]::Round(408 * $scale)
        $radius = [int][Math]::Round(12 * $scale)
        Save-Panel $hwnd "droplet-core-$percent.png" $dpiOut $width $height $radius
        Write-Output "Captured DPI $percent% at ${width}x${height}"
    }

    # Return to 100% for the native-size site images after the DPI path has
    # settled. These are the four canonical captures used by the site and
    # README.
    # A layered window at the same 624x408 size as a previous instance can
    # leave DWM with a cached surface; PrintWindow may then return that stale
    # surface even though the app has drawn the new one. Resize through a
    # different physical size and back so the final 100% image is composed
    # from the current surface, not from the previous window at that size.
    $beforeReset = New-Object DropletTabs+RECT
    [DropletTabs]::GetWindowRect($hwnd, [ref]$beforeReset) | Out-Null
    [DropletTabs]::SetWindowPos($hwnd, [IntPtr]::Zero, $beforeReset.Left, $beforeReset.Top, 625, 409, 0x0004 -bor 0x0010) | Out-Null
    Start-Sleep -Milliseconds 300
    Set-PanelDpi $hwnd 100
    foreach ($tab in $tabs) {
        Click-Tab $hwnd ([int]$tab.X) ([int]$tab.Y)
        Save-Panel $hwnd $tab.Name $out 624 408 12
        Write-Output "Captured $($tab.Name)"
    }
} finally {
    if ($null -ne $process -and -not $process.HasExited) {
        Stop-Process -Id $process.Id -PassThru | Wait-Process -Timeout 5 -ErrorAction SilentlyContinue
    }
    foreach ($name in $savedEnv.Keys) {
        if ($null -eq $savedEnv[$name]) {
            Remove-Item "Env:$name" -ErrorAction SilentlyContinue
        } else {
            Set-Item "Env:$name" $savedEnv[$name]
        }
    }
    Remove-Item -LiteralPath $captureRoot -Recurse -Force -ErrorAction SilentlyContinue
}
