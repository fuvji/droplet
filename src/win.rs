//! Minimal Win32 FFI surface. No external crates: everything the app needs is
//! declared here (user32/kernel32/gdi32 are linked statically, the optional
//! DLLs - gdiplus, dwmapi, winmm - are loaded at runtime).

#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::c_void;

pub type HWND = isize;
pub type HINSTANCE = isize;
pub type HDC = isize;
pub type HBITMAP = isize;
pub type HGDIOBJ = isize;
pub type HMENU = isize;
pub type HICON = isize;
pub type HMODULE = isize;
pub type HANDLE = isize;
pub type HBRUSH = isize;
pub type HCURSOR = isize;
pub type HRGN = isize;
pub type HHOOK = isize;
pub type LRESULT = isize;
pub type LPARAM = isize;
pub type WPARAM = usize;
pub type ATOM = u16;
pub type BOOL = i32;
pub type DWORD = u32;
pub type UINT = u32;

pub type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;
pub type HookProc = unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT;
pub type EnumProc = unsafe extern "system" fn(HWND, LPARAM) -> BOOL;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl RECT {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
    pub lPrivate: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

#[repr(C)]
pub struct SHFILEOPSTRUCTW {
    pub hwnd: HWND,
    pub w_func: u32,
    pub p_from: *const u16,
    pub p_to: *const u16,
    pub flags: u16,
    pub aborted: i32,
    pub name_mappings: *mut c_void,
    pub progress_title: *const u16,
}

/// NOTIFYICONDATAW (V3 layout, 976 bytes on x64).
#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: u32,
    pub uFlags: u32,
    pub uCallbackMessage: u32,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub uVersion: u32,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: GUID,
    pub hBalloonIcon: HICON,
}

#[repr(C)]
pub struct AccentPolicy {
    pub state: i32,
    pub flags: i32,
    pub gradient: u32,
    pub animation: i32,
}

#[repr(C)]
pub struct WindowCompositionAttributeData {
    pub attrib: i32,
    pub data: *mut c_void,
    pub size: usize,
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: usize,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: HICON,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KBDLLHOOKSTRUCT {
    pub vkCode: u32,
    pub scanCode: u32,
    pub flags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSLLHOOKSTRUCT {
    pub pt: POINT,
    pub mouseData: u32,
    pub flags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MOUSEINPUT {
    pub dx: i32,
    pub dy: i32,
    pub mouseData: u32,
    pub dwFlags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KEYBDINPUT {
    pub wVk: u16,
    pub wScan: u16,
    pub dwFlags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HARDWAREINPUT {
    pub uMsg: u32,
    pub wParamL: u16,
    pub wParamH: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union INPUTUNION {
    pub mi: MOUSEINPUT,
    pub ki: KEYBDINPUT,
    pub hi: HARDWAREINPUT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct INPUT {
    pub r#type: u32,
    pub u: INPUTUNION,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BITMAPINFOHEADER {
    pub biSize: u32,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: u32,
    pub biSizeImage: u32,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: u32,
    pub biClrImportant: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [u32; 1],
}

// ---------------------------------------------------------------- constants

pub const WM_NULL: u32 = 0x0000;
pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_SETCURSOR: u32 = 0x0020;
pub const WM_MOUSEACTIVATE: u32 = 0x0021;
pub const WM_GETMINMAXINFO: u32 = 0x0024;
pub const WM_NCCREATE: u32 = 0x0081;
pub const WM_NCHITTEST: u32 = 0x0084;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_SYSKEYDOWN: u32 = 0x0104;
pub const WM_SYSKEYUP: u32 = 0x0105;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_CAPTURECHANGED: u32 = 0x0215;
pub const WM_APP_MODULE_TOGGLE: u32 = 0x8F01;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_RBUTTONDOWN: u32 = 0x0204;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const WM_MBUTTONDOWN: u32 = 0x0207;
pub const WM_MBUTTONUP: u32 = 0x0208;
pub const WM_MOUSEWHEEL: u32 = 0x020A;
pub const WM_DPICHANGED: u32 = 0x02E0;
pub const WM_APP: u32 = 0x8000;
pub const WM_APP_CAPTURED: u32 = WM_APP + 1;
pub const WM_APP_STATE: u32 = WM_APP + 2;
pub const WM_APP_TICK: u32 = WM_APP + 3;
pub const WM_APP_PANEL: u32 = WM_APP + 4;
pub const WM_APP_MASTER: u32 = WM_APP + 5;
pub const WM_APP_DESTRUCT: u32 = WM_APP + 6;
pub const WM_APP_TRAY: u32 = WM_APP + 7;

pub const NIM_ADD: u32 = 0;
pub const NIM_MODIFY: u32 = 1;
pub const NIM_DELETE: u32 = 2;

pub const FO_DELETE: u32 = 3;
pub const FOF_SILENT: u16 = 0x0004;
pub const FOF_NOCONFIRMATION: u16 = 0x0010;
pub const FOF_ALLOWUNDO: u16 = 0x0040;
pub const FOF_NOERRORUI: u16 = 0x0400;
pub const NIF_MESSAGE: u32 = 0x0000_0001;
pub const NIF_ICON: u32 = 0x0000_0002;
pub const NIF_TIP: u32 = 0x0000_0004;

pub const MF_STRING: u32 = 0x0000_0000;
pub const MF_SEPARATOR: u32 = 0x0000_0800;
pub const TPM_RIGHTBUTTON: u32 = 0x0000_0002;
pub const TPM_BOTTOMALIGN: u32 = 0x0000_0020;
pub const TPM_RETURNCMD: u32 = 0x0000_0100;

pub const WS_POPUP: u32 = 0x8000_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const WS_EX_TOPMOST: u32 = 0x0000_0008;
pub const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
pub const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
pub const WS_EX_LAYERED: u32 = 0x0008_0000;

pub const LWA_ALPHA: u32 = 0x0000_0002;
pub const WCA_ACCENT_POLICY: i32 = 19;
pub const ACCENT_ENABLE_ACRYLICBLURBEHIND: i32 = 4;

pub const SW_HIDE: i32 = 0;
pub const SW_SHOWNOACTIVATE: i32 = 4;
pub const SW_SHOW: i32 = 5;

pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_SHOWWINDOW: u32 = 0x0040;

pub const HWND_TOPMOST: HWND = -1;
pub const HTCAPTION: LRESULT = 2;
pub const HTCLIENT: LRESULT = 1;
pub const HTTRANSPARENT: LRESULT = -1;

pub const WM_MOUSEACTIVATE_NOACTIVATE: LRESULT = 3; // MA_NOACTIVATE

pub const WH_KEYBOARD_LL: i32 = 13;
pub const WH_MOUSE_LL: i32 = 14;

pub const INPUT_MOUSE: u32 = 0;
pub const INPUT_KEYBOARD: u32 = 1;

pub const MOUSEEVENTF_MOVE: u32 = 0x0001;
pub const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
pub const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
pub const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
pub const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
pub const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
pub const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
pub const MOUSEEVENTF_XDOWN: u32 = 0x0080;
pub const MOUSEEVENTF_XUP: u32 = 0x0100;

pub const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
pub const KEYEVENTF_KEYUP: u32 = 0x0002;

pub const MAPVK_VK_TO_VSC: u32 = 0;
pub const MAPVK_VK_TO_VSC_EX: u32 = 4;

pub const GWLP_USERDATA: i32 = -21;


pub const SPI_GETWORKAREA: u32 = 0x0030;

pub const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

pub const GA_ROOT: u32 = 2;
pub const THREAD_PRIORITY_NORMAL: i32 = 0;
pub const THREAD_PRIORITY_TIME_CRITICAL: i32 = 15;
pub const THREAD_SET_INFORMATION: u32 = 0x0020;

pub const NORMAL_PRIORITY_CLASS: u32 = 0x0000_0020;
pub const REALTIME_PRIORITY_CLASS: u32 = 0x0000_0100;
pub const HIGH_PRIORITY_CLASS: u32 = 0x0000_0080;

pub const SRCCOPY: u32 = 0x00CC_0020;

pub const DIB_RGB_COLORS: u32 = 0;

pub const ERROR_ALREADY_EXISTS: u32 = 183;

pub const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4;

pub const IDC_ARROW: usize = 32512;

// ---------------------------------------------------------------- imports

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassExW(wc: *const WNDCLASSEXW) -> ATOM;
    pub fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: HWND,
        menu: HMENU,
        inst: HINSTANCE,
        param: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT;
    pub fn DestroyWindow(hwnd: HWND) -> BOOL;
    pub fn ShowWindow(hwnd: HWND, cmd: i32) -> BOOL;
    pub fn SetWindowPos(hwnd: HWND, after: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> BOOL;
    pub fn GetWindowRect(hwnd: HWND, r: *mut RECT) -> BOOL;
    pub fn GetClientRect(hwnd: HWND, r: *mut RECT) -> BOOL;
    pub fn SetWindowRgn(hwnd: HWND, rgn: HRGN, redraw: BOOL) -> i32;
    pub fn GetMessageW(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> BOOL;
    pub fn TranslateMessage(msg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(msg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(code: i32);
    pub fn PostMessageW(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> BOOL;
    pub fn SetTimer(hwnd: HWND, id: usize, ms: u32, cb: usize) -> usize;
    pub fn KillTimer(hwnd: HWND, id: usize) -> BOOL;
    pub fn InvalidateRect(hwnd: HWND, r: *const RECT, erase: BOOL) -> BOOL;
    pub fn GetDC(hwnd: HWND) -> HDC;
    pub fn ReleaseDC(hwnd: HWND, hdc: HDC) -> i32;
    pub fn BeginPaint(hwnd: HWND, ps: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hwnd: HWND, ps: *const PAINTSTRUCT) -> BOOL;
    pub fn SetCursor(cur: HCURSOR) -> HCURSOR;
    pub fn LoadCursorW(inst: HINSTANCE, name: *const u16) -> HCURSOR;
    pub fn GetCursorPos(p: *mut POINT) -> BOOL;
    pub fn GetMessageExtraInfo() -> isize;
    pub fn SetCapture(hwnd: HWND) -> HWND;
    pub fn ReleaseCapture() -> BOOL;
    pub fn SetErrorMode(mode: u32) -> u32;
    pub fn ScreenToClient(hwnd: HWND, p: *mut POINT) -> BOOL;
    pub fn ClientToScreen(hwnd: HWND, p: *mut POINT) -> BOOL;
    pub fn GetAsyncKeyState(vk: i32) -> i16;
    pub fn MapVirtualKeyW(code: u32, maptype: u32) -> u32;
    pub fn SetWindowsHookExW(id: i32, proc: usize, inst: HINSTANCE, tid: u32) -> HHOOK;
    pub fn UnhookWindowsHookEx(hh: HHOOK) -> BOOL;
    pub fn CallNextHookEx(hh: HHOOK, code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT;
    pub fn GetForegroundWindow() -> HWND;
    pub fn GetWindowThreadProcessId(hwnd: HWND, pid: *mut u32) -> u32;
    pub fn IsWindow(hwnd: HWND) -> BOOL;
    pub fn IsWindowVisible(hwnd: HWND) -> BOOL;
    pub fn GetClassNameW(hwnd: HWND, buf: *mut u16, max: i32) -> i32;
    pub fn GetWindowTextW(hwnd: HWND, buf: *mut u16, max: i32) -> i32;
    pub fn GetWindowTextLengthW(hwnd: HWND) -> i32;
    pub fn GetWindow(hwnd: HWND, cmd: u32) -> HWND;
    pub fn GetAncestor(hwnd: HWND, flags: u32) -> HWND;
    pub fn EnumWindows(cb: EnumProc, lp: LPARAM) -> BOOL;
    pub fn GetWindowLongPtrW(hwnd: HWND, idx: i32) -> isize;
    pub fn SetWindowLongPtrW(hwnd: HWND, idx: i32, val: isize) -> isize;
    pub fn SetLayeredWindowAttributes(hwnd: HWND, key: u32, alpha: u8, flags: u32) -> BOOL;
    pub fn SendInput(count: u32, inputs: *const INPUT, size: i32) -> u32;
    pub fn SetProcessDpiAwarenessContext(ctx: isize) -> BOOL;
    pub fn GetDpiForWindow(hwnd: HWND) -> u32;
    pub fn SystemParametersInfoW(action: u32, p: u32, pv: *mut c_void, winini: u32) -> BOOL;
    pub fn SetWindowTextW(hwnd: HWND, s: *const u16) -> BOOL;
    pub fn GetSystemMetrics(idx: i32) -> i32;
    pub fn SetFocus(hwnd: HWND) -> HWND;
    pub fn CreateIcon(hinst: HINSTANCE, w: i32, h: i32, planes: u8, bits: u8, and_mask: *const u8, xor_mask: *const u8) -> HICON;
    pub fn SetClassLongPtrW(hwnd: HWND, idx: i32, val: isize) -> isize;
    pub fn SetForegroundWindow(hwnd: HWND) -> BOOL;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn AppendMenuW(menu: HMENU, flags: u32, id: usize, text: *const u16) -> BOOL;
    pub fn DestroyMenu(menu: HMENU) -> BOOL;
    pub fn TrackPopupMenu(menu: HMENU, flags: u32, x: i32, y: i32, reserved: i32, hwnd: HWND, rect: *const RECT) -> BOOL;
    pub fn RegisterWindowMessageW(name: *const u16) -> u32;
    pub fn DestroyIcon(icon: HICON) -> BOOL;
}

#[link(name = "shell32")]
extern "system" {
    pub fn Shell_NotifyIconW(msg: u32, data: *const NOTIFYICONDATAW) -> BOOL;
    pub fn SHFileOperationW(op: *mut SHFILEOPSTRUCTW) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name: *const u16) -> HMODULE;
    pub fn LoadLibraryW(name: *const u16) -> HMODULE;
    pub fn GetProcAddress(module: HMODULE, name: *const u8) -> usize;
    pub fn GetModuleFileNameW(module: HMODULE, buf: *mut u16, size: u32) -> u32;
    pub fn QueryPerformanceCounter(v: *mut i64) -> BOOL;
    pub fn QueryPerformanceFrequency(v: *mut i64) -> BOOL;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn GetCurrentThread() -> HANDLE;
    pub fn OpenProcess(access: u32, inherit: BOOL, pid: u32) -> HANDLE;
    pub fn QueryFullProcessImageNameW(h: HANDLE, flags: u32, buf: *mut u16, size: *mut u32) -> BOOL;
    pub fn CloseHandle(h: HANDLE) -> BOOL;
    pub fn OpenThread(access: u32, inherit: BOOL, tid: u32) -> HANDLE;
    pub fn GetCurrentThreadId() -> u32;
    pub fn SetPriorityClass(proc: HANDLE, class: u32) -> BOOL;
    pub fn SetThreadPriority(thread: HANDLE, prio: i32) -> BOOL;
    pub fn GetLastError() -> u32;
    pub fn SetLastError(err: u32);
    pub fn Sleep(ms: u32);
    pub fn CreateMutexW(attrs: *mut c_void, initial: BOOL, name: *const u16) -> HANDLE;
    pub fn GetTickCount64() -> u64;
    pub fn ProcessIdToSessionId(pid: u32, session: *mut u32) -> BOOL;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn CreateCompatibleBitmap(hdc: HDC, w: i32, h: i32) -> HBITMAP;
    pub fn CreateDIBSection(hdc: HDC, pbmi: *const BITMAPINFO, usage: u32, bits: *mut *mut c_void, section: HANDLE, offset: u32) -> HBITMAP;
    pub fn SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(obj: HGDIOBJ) -> BOOL;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn BitBlt(dst: HDC, x: i32, y: i32, w: i32, h: i32, src: HDC, sx: i32, sy: i32, rop: u32) -> BOOL;
    pub fn CreateRoundRectRgn(x1: i32, y1: i32, x2: i32, y2: i32, w: i32, h: i32) -> HRGN;
    pub fn CreateSolidBrush(color: u32) -> HBRUSH;
    pub fn FillRect(hdc: HDC, r: *const RECT, brush: HBRUSH) -> i32;
}

// ---------------------------------------------------------------- helpers

pub fn wide(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

pub fn wide_buf(s: &str, buf: &mut [u16]) {
    let mut i = 0;
    for u in s.encode_utf16() {
        if i + 1 >= buf.len() {
            break;
        }
        buf[i] = u;
        i += 1;
    }
    buf[i] = 0;
}

pub unsafe fn get_class(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

pub unsafe fn get_title(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

pub unsafe fn process_image(pid: u32) -> String {
    let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if h == 0 {
        return String::new();
    }
    let mut buf = [0u16; 1024];
    let mut size = buf.len() as u32;
    let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut size);
    CloseHandle(h);
    if ok == 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..size as usize])
}

pub unsafe fn window_pid(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, &mut pid);
    pid
}

pub unsafe fn is_down(vk: u32) -> bool {
    (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0
}

pub fn input_size() -> i32 {
    std::mem::size_of::<INPUT>() as i32
}

pub fn load_proc(module: HMODULE, name: &str) -> usize {
    // GetProcAddress takes an ANSI name, not UTF-16. A stack buffer keeps this
    // allocation-free; every dynamic name used here is well under 64 bytes.
    let bytes = name.as_bytes();
    if bytes.len() >= 64 {
        return 0;
    }
    let mut buf = [0u8; 64];
    buf[..bytes.len()].copy_from_slice(bytes);
    unsafe { GetProcAddress(module, buf.as_ptr()) }
}

/// Send one absolute path to the Recycle Bin. Windows refuses this for a
/// running image, so the caller must wait/recycle externally in that case.
pub fn recycle(path: &std::path::Path) -> bool {
    use std::os::windows::ffi::OsStrExt;
    if !path.is_absolute() {
        return false;
    }
    let mut from: Vec<u16> = path.as_os_str().encode_wide().collect();
    if from.is_empty() {
        return false;
    }
    from.extend_from_slice(&[0, 0]);
    let mut op = SHFILEOPSTRUCTW {
        hwnd: 0,
        w_func: FO_DELETE,
        p_from: from.as_ptr(),
        p_to: std::ptr::null(),
        flags: FOF_SILENT | FOF_NOCONFIRMATION | FOF_ALLOWUNDO | FOF_NOERRORUI,
        aborted: 0,
        name_mappings: std::ptr::null_mut(),
        progress_title: std::ptr::null(),
    };
    unsafe { SHFileOperationW(&mut op) == 0 && op.aborted == 0 }
}

pub fn dyn_load(dll: &str) -> HMODULE {
    let w = wide(dll);
    unsafe { LoadLibraryW(w.as_ptr()) }
}
