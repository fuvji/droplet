#Requires -Version 5.1
<#
.SYNOPSIS
    Record Droplet's injected input and verify focus/master gate behavior.

.DESCRIPTION
    This is a local integration harness, not part of the shipped panel. It
    installs low-level keyboard and mouse hooks in a hidden Windows message
    loop, starts Droplet with a temporary profile, sends one labelled trigger,
    and records both physical and injected events. It then asserts that only
    ordinary SendInput sequences are emitted and that the focus/master gates
    suppress them when they should.

    The harness runs in an interactive Windows session. It does not need a
    Minecraft client, administrator rights, game memory, or network access.

    The trigger is intentionally tagged with a private dwExtraInfo value. The
    Droplet hook accepts it as physical input; the recorder ignores it so the
    recorded sequence contains only Droplet's compiled macro output.

.PARAMETER SettleMs
    Time to wait after launching Droplet for its hooks and gate watcher.

.PARAMETER OutFile
    Optional JSON path for the complete per-case event record.

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File tools/input-harness.ps1

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File tools/input-harness.ps1 -OutFile "$env:TEMP\\droplet-input.json"
#>
param(
    [int]$SettleMs = 1800,
    [string]$OutFile = ""
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "target\x86_64-pc-windows-gnu\release\droplet.exe"
if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
    throw "Build the release executable before running the input harness: $exe"
}

$dropletMarker = [uint64]0x48594452   # input::INJ ("HYDR")
$harnessMarker = [uint64]0x48524E53   # "HRNS"

Add-Type -AssemblyName System.Windows.Forms
Add-Type -ReferencedAssemblies System.Windows.Forms, System.Core -TypeDefinition @"
using System;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading;
using System.Windows.Forms;

public sealed class HarnessInput
{
    public string Kind;
    public int Code;
    public bool Down;
    public bool Injected;
    public UInt64 Extra;
    public long Micros;

    public HarnessInput(string kind, int code, bool down, bool injected, UInt64 extra, long micros)
    {
        Kind = kind;
        Code = code;
        Down = down;
        Injected = injected;
        Extra = extra;
        Micros = micros;
    }
}

public static class InputHarness
{
    private const int WH_KEYBOARD_LL = 13;
    private const int WH_MOUSE_LL = 14;
    private const int WM_KEYDOWN = 0x0100;
    private const int WM_KEYUP = 0x0101;
    private const int WM_SYSKEYDOWN = 0x0104;
    private const int WM_SYSKEYUP = 0x0105;
    private const int WM_LBUTTONDOWN = 0x0201;
    private const int WM_LBUTTONUP = 0x0202;
    private const int WM_RBUTTONDOWN = 0x0204;
    private const int WM_RBUTTONUP = 0x0205;
    private const int WM_MBUTTONDOWN = 0x0207;
    private const int WM_MBUTTONUP = 0x0208;
    private const int WM_XBUTTONDOWN = 0x020B;
    private const int WM_XBUTTONUP = 0x020C;
    private const uint INPUT_KEYBOARD = 1;
    private const uint KEYEVENTF_KEYUP = 0x0002;
    private const uint WM_QUIT = 0x0012;

    public const UInt64 HarnessMarker = 0x48524E53;

    [StructLayout(LayoutKind.Sequential)]
    private struct KBDLLHOOKSTRUCT
    {
        public uint vkCode;
        public uint scanCode;
        public uint flags;
        public uint time;
        public UIntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct MSLLHOOKSTRUCT
    {
        public int ptX;
        public int ptY;
        public uint mouseData;
        public uint flags;
        public uint time;
        public UIntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct KEYBDINPUT
    {
        public ushort wVk;
        public ushort wScan;
        public uint dwFlags;
        public uint time;
        public UIntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct MOUSEINPUT
    {
        public int dx;
        public int dy;
        public uint mouseData;
        public uint dwFlags;
        public uint time;
        public UIntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Explicit)]
    private struct INPUTUNION
    {
        [FieldOffset(0)] public MOUSEINPUT mi;
        [FieldOffset(0)] public KEYBDINPUT ki;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct INPUT
    {
        public uint type;
        public INPUTUNION u;
    }

    private delegate IntPtr LowLevelProc(int nCode, IntPtr wParam, IntPtr lParam);

    private static readonly ConcurrentQueue<HarnessInput> Queue = new ConcurrentQueue<HarnessInput>();
    private static readonly Stopwatch Clock = Stopwatch.StartNew();
    private static LowLevelProc kbProc;
    private static LowLevelProc msProc;
    private static IntPtr kbHook;
    private static IntPtr msHook;
    private static Thread worker;
    private static ManualResetEventSlim ready;
    private static int workerId;
    private static string lastError;

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetWindowsHookEx(int idHook, LowLevelProc lpfn, IntPtr hMod, uint dwThreadId);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool UnhookWindowsHookEx(IntPtr hhk);

    [DllImport("user32.dll")]
    private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("kernel32.dll")]
    private static extern IntPtr GetModuleHandle(string name);

    [DllImport("kernel32.dll")]
    private static extern uint GetCurrentThreadId();

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool PostThreadMessage(uint idThread, uint msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

    private static void Add(string kind, int code, bool down, bool injected, UInt64 extra)
    {
        Queue.Enqueue(new HarnessInput(kind, code, down, injected, extra, Clock.ElapsedTicks * 1000000L / Stopwatch.Frequency));
    }

    private static IntPtr KeyboardProc(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0)
        {
            KBDLLHOOKSTRUCT info = (KBDLLHOOKSTRUCT)Marshal.PtrToStructure(lParam, typeof(KBDLLHOOKSTRUCT));
            UInt64 extra = info.dwExtraInfo.ToUInt64();
            int msg = wParam.ToInt32();
            bool down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            bool up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
            if ((down || up) && extra != HarnessMarker)
            {
                Add("key", (int)info.vkCode, down, (info.flags & 0x10) != 0, extra);
            }
        }
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
    }

    private static IntPtr MouseProc(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0)
        {
            MSLLHOOKSTRUCT info = (MSLLHOOKSTRUCT)Marshal.PtrToStructure(lParam, typeof(MSLLHOOKSTRUCT));
            UInt64 extra = info.dwExtraInfo.ToUInt64();
            int msg = wParam.ToInt32();
            int code = 0;
            bool down = false;
            bool known = true;
            switch (msg)
            {
                case WM_LBUTTONDOWN: code = 1; down = true; break;
                case WM_LBUTTONUP: code = 1; down = false; break;
                case WM_RBUTTONDOWN: code = 2; down = true; break;
                case WM_RBUTTONUP: code = 2; down = false; break;
                case WM_MBUTTONDOWN: code = 3; down = true; break;
                case WM_MBUTTONUP: code = 3; down = false; break;
                case WM_XBUTTONDOWN: code = (int)(info.mouseData >> 16); down = true; break;
                case WM_XBUTTONUP: code = (int)(info.mouseData >> 16); down = false; break;
                default: known = false; break;
            }
            if (known && extra != HarnessMarker)
            {
                Add("mouse", code, down, (info.flags & 0x01) != 0, extra);
            }
        }
        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
    }

    private sealed class MessageWindow : Form
    {
        protected override void SetVisibleCore(bool value)
        {
            base.SetVisibleCore(false);
        }
    }

    public static void Start()
    {
        if (worker != null)
        {
            return;
        }
        ready = new ManualResetEventSlim(false);
        lastError = null;
        worker = new Thread(delegate()
        {
            try
            {
                workerId = (int)GetCurrentThreadId();
                kbProc = KeyboardProc;
                msProc = MouseProc;
                kbHook = SetWindowsHookEx(WH_KEYBOARD_LL, kbProc, GetModuleHandle(null), 0);
                msHook = SetWindowsHookEx(WH_MOUSE_LL, msProc, GetModuleHandle(null), 0);
                if (kbHook == IntPtr.Zero || msHook == IntPtr.Zero)
                {
                    throw new InvalidOperationException("SetWindowsHookEx failed.");
                }
                ready.Set();
                Application.Run(new MessageWindow());
            }
            catch (Exception ex)
            {
                lastError = ex.Message;
                ready.Set();
            }
            finally
            {
                if (kbHook != IntPtr.Zero) UnhookWindowsHookEx(kbHook);
                if (msHook != IntPtr.Zero) UnhookWindowsHookEx(msHook);
                kbHook = IntPtr.Zero;
                msHook = IntPtr.Zero;
            }
        });
        worker.IsBackground = true;
        worker.SetApartmentState(ApartmentState.STA);
        worker.Start();
        if (!ready.Wait(5000))
        {
            throw new InvalidOperationException("Input recorder did not initialize within 5 seconds.");
        }
        if (lastError != null)
        {
            throw new InvalidOperationException("Input recorder failed: " + lastError);
        }
    }

    public static void Stop()
    {
        if (worker == null)
        {
            return;
        }
        PostThreadMessage((uint)workerId, WM_QUIT, IntPtr.Zero, IntPtr.Zero);
        worker.Join(3000);
        worker = null;
    }

    public static void Clear()
    {
        HarnessInput ignored;
        while (Queue.TryDequeue(out ignored)) { }
    }

    public static HarnessInput[] Snapshot()
    {
        return Queue.ToArray();
    }

    public static void TapKey(ushort vk, int holdMs)
    {
        SendKey(vk, false);
        Thread.Sleep(Math.Max(1, holdMs));
        SendKey(vk, true);
    }

    private static void SendKey(ushort vk, bool up)
    {
        INPUT input = new INPUT();
        input.type = INPUT_KEYBOARD;
        input.u.ki.wVk = vk;
        input.u.ki.wScan = 0;
        input.u.ki.dwFlags = up ? KEYEVENTF_KEYUP : 0;
        input.u.ki.time = 0;
        input.u.ki.dwExtraInfo = (UIntPtr)HarnessMarker;
        INPUT[] batch = new INPUT[] { input };
        uint sent = SendInput(1, batch, Marshal.SizeOf(typeof(INPUT)));
        if (sent != 1)
        {
            throw new InvalidOperationException("SendInput failed. Win32 error: " + Marshal.GetLastWin32Error().ToString());
        }
    }
}
"@

function Write-TestProfile([string]$DataDir, [int]$FocusOnly, [int]$Master) {
    $config = @"
[T]
v0=$Master
vD=0
v1=1
v2=0
v3=0
v4=0
v6=0
v5=0
v7=0
v8=0
v9=0
vA=0
vB=0
vC=0
vF=0
vW=0
vG=0
vP=0
vI=0
vFO=$FocusOnly
v1Bn=q
v1S1=h
v1Md=hold
"@
    Set-Content -LiteralPath (Join-Path $DataDir "config.ini") -Encoding ASCII -Value $config
}

function Invoke-Case([string]$Name, [int]$FocusOnly, [int]$Master) {
    $caseRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("droplet-input-" + [Guid]::NewGuid().ToString("N"))
    $dataDir = Join-Path $caseRoot "Droplet"
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null
    Write-TestProfile $dataDir $FocusOnly $Master

    $savedEnv = @{
        LOCALAPPDATA = $env:LOCALAPPDATA
        TEMP = $env:TEMP
        TMP = $env:TMP
    }
    $process = $null
    try {
        $env:LOCALAPPDATA = $caseRoot
        $env:TEMP = $caseRoot
        $env:TMP = $caseRoot
        $process = Start-Process -FilePath $exe -WorkingDirectory $root -PassThru
        Start-Sleep -Milliseconds $SettleMs
        if ($process.HasExited) {
            throw "Droplet exited before the harness could trigger it."
        }

        [InputHarness]::Clear()
        [InputHarness]::TapKey(0x51, 20) # 'q' = Shield trigger in the test profile
        Start-Sleep -Milliseconds 700
        $events = [InputHarness]::Snapshot()
        $output = @($events | Where-Object { [uint64]$_.Extra -eq $dropletMarker })

        $keySeq = @($output | Where-Object { $_.Kind -eq "key" } | ForEach-Object { "{0}:{1}" -f $_.Code, $(if ($_.Down) { "down" } else { "up" }) })
        $mouseSeq = @($output | Where-Object { $_.Kind -eq "mouse" } | ForEach-Object { "{0}:{1}" -f $_.Code, $(if ($_.Down) { "down" } else { "up" }) })
        $expectedShield = ($keySeq.Count -eq 2 -and $keySeq[0] -eq "72:down" -and $keySeq[1] -eq "72:up" -and
                           $mouseSeq.Count -eq 2 -and $mouseSeq[0] -eq "1:down" -and $mouseSeq[1] -eq "1:up")
        $expectsOutput = ($Master -eq 1 -and $FocusOnly -eq 0)
        $passed = if ($expectsOutput) { $expectedShield } else { $output.Count -eq 0 }

        [pscustomobject]@{
            Case = $Name
            FocusOnly = $FocusOnly
            Master = $Master
            Expected = $(if ($expectsOutput) { "H down/up + Left down/up" } else { "no macro output" })
            Passed = $passed
            KeySequence = ($keySeq -join ", ")
            MouseSequence = ($mouseSeq -join ", ")
            OutputEvents = $output.Count
            Events = $events
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
        Remove-Item -LiteralPath $caseRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}

$results = @()
[InputHarness]::Start()
try {
    $results += Invoke-Case "focus-gate" 1 1
    $results += Invoke-Case "master-off" 0 0
    $results += Invoke-Case "shield-sequence" 0 1
} finally {
    [InputHarness]::Stop()
}

if ($OutFile -ne "") {
    $results | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $OutFile -Encoding UTF8
}

foreach ($result in $results) {
    $state = if ($result.Passed) { "PASS" } else { "FAIL" }
    Write-Output ("{0}  {1}: keys=[{2}] mouse=[{3}] output={4}" -f $state, $result.Case, $result.KeySequence, $result.MouseSequence, $result.OutputEvents)
}

$failed = @($results | Where-Object { -not $_.Passed })
if ($failed.Count -gt 0) {
    Write-Output ("INPUT HARNESS FAILED: {0} case(s)" -f $failed.Count)
    exit 1
}
Write-Output "INPUT HARNESS PASSED: focus gate, master gate, and Shield sequence."
exit 0
