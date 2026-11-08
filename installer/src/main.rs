#![windows_subsystem = "windows"]

use std::env;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{BOOL, COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
    FillRect, GetStockObject, InvalidateRect, MoveToEx, LineTo, RoundRect, SelectObject, SetBkColor,
    SetBkMode, SetTextColor, TextOutW, DT_CENTER, DT_SINGLELINE, DT_VCENTER, FW_BOLD, FW_NORMAL,
    FW_SEMIBOLD, HBRUSH, HDC, NULL_PEN, PAINTSTRUCT, PS_SOLID, TRANSPARENT,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, IPersistFile,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{
    InitCommonControlsEx, DRAWITEMSTRUCT, INITCOMMONCONTROLSEX, INITCOMMONCONTROLSEX_ICC,
    ODS_SELECTED, PROGRESS_CLASS,
};
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, DrawIconEx, GetClientRect,
    GetMessageW, GetSystemMetrics, LoadIconW, MessageBoxW, PeekMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, SetWindowTextW, ShowWindow, TranslateMessage, BM_GETCHECK,
    BM_SETCHECK, BS_AUTOCHECKBOX, BS_OWNERDRAW, CS_HREDRAW, CS_VREDRAW, DI_NORMAL, ES_AUTOHSCROLL,
    ES_READONLY, HCURSOR, HICON, IDI_APPLICATION, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION,
    MB_OK, MB_YESNO, MSG, PM_REMOVE, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_COMMAND, WM_CTLCOLORBTN, WM_CTLCOLOREDIT, WM_CTLCOLORSTATIC, WM_DESTROY,
    WM_DRAWITEM, WM_PAINT, WNDCLASSEXW, WS_CHILD, WS_CLIPCHILDREN, WS_MINIMIZEBOX, WS_OVERLAPPED,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
};

const CREATE_NO_WINDOW: u32 = 0x08000000;
const WS_CAPTION: u32 = 0x00C00000;
const BST_CHECKED_VAL: usize = 1;
const PBM_SETRANGE32: u32 = 1030; // WM_USER + 6
const PBM_SETPOS: u32 = 1026; // WM_USER + 2

// Embedded Payload Files
struct EmbeddedFile {
    path: &'static str,
    bytes: &'static [u8],
}

const EMBEDDED_FILES: &[EmbeddedFile] = &[
    EmbeddedFile {
        path: "Nexora.exe",
        bytes: include_bytes!("../../target/release/Nexora.exe"),
    },
    EmbeddedFile {
        path: "README.md",
        bytes: include_bytes!("../../README.md"),
    },
    EmbeddedFile {
        path: "LICENSE",
        bytes: include_bytes!("../../LICENSE"),
    },
    EmbeddedFile {
        path: "CHANGELOG.md",
        bytes: include_bytes!("../../CHANGELOG.md"),
    },
    EmbeddedFile {
        path: "assets\\branding\\icon.ico",
        bytes: include_bytes!("../../assets/branding/icon.ico"),
    },
    EmbeddedFile {
        path: "assets\\branding\\icon_256.png",
        bytes: include_bytes!("../../assets/branding/icon_256.png"),
    },
    EmbeddedFile {
        path: "assets\\branding\\app_icon.png",
        bytes: include_bytes!("../../assets/branding/app_icon.png"),
    },
    EmbeddedFile {
        path: "assets\\branding\\app_icon_black.png",
        bytes: include_bytes!("../../assets/branding/app_icon_black.png"),
    },
    EmbeddedFile {
        path: "assets\\branding\\app_icon_b64.txt",
        bytes: include_bytes!("../../assets/branding/app_icon_b64.txt"),
    },
    EmbeddedFile {
        path: "assets\\branding\\icon.svg",
        bytes: include_bytes!("../../assets/branding/icon.svg"),
    },
    EmbeddedFile {
        path: "assets\\branding\\logo.svg",
        bytes: include_bytes!("../../assets/branding/logo.svg"),
    },
    EmbeddedFile {
        path: "assets\\branding\\chrome_icons_atlas.png",
        bytes: include_bytes!("../../assets/branding/chrome_icons_atlas.png"),
    },
    EmbeddedFile {
        path: "assets\\branding\\chrome_icons_atlas.bin",
        bytes: include_bytes!("../../assets/branding/chrome_icons_atlas.bin"),
    },
];

const ID_BTN_INSTALL: usize = 1001;
const ID_BTN_CANCEL: usize = 1002;
const ID_CHK_DESKTOP: usize = 1003;
const ID_CHK_START: usize = 1004;
const ID_CHK_LAUNCH: usize = 1005;
const ID_PROGRESS: usize = 1006;
const ID_STATUS: usize = 1007;
const ID_PATH_EDIT: usize = 1008;

static IS_FINISHED: AtomicBool = AtomicBool::new(false);

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn get_default_install_dir() -> PathBuf {
    if let Ok(local) = env::var("LOCALAPPDATA") {
        PathBuf::from(local).join("Programs").join("Nexora")
    } else if let Ok(user) = env::var("USERPROFILE") {
        PathBuf::from(user).join("AppData").join("Local").join("Programs").join("Nexora")
    } else {
        PathBuf::from("C:\\Program Files\\Nexora")
    }
}

fn get_desktop_shortcut_path() -> Option<PathBuf> {
    if let Ok(user) = env::var("USERPROFILE") {
        Some(PathBuf::from(user).join("Desktop").join("Nexora.lnk"))
    } else {
        None
    }
}

fn get_start_menu_shortcut_path() -> Option<PathBuf> {
    if let Ok(appdata) = env::var("APPDATA") {
        Some(
            PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Nexora.lnk"),
        )
    } else {
        None
    }
}

fn run_command_hidden(cmd: &str, args: &[&str]) -> bool {
    let mut command = Command::new(cmd);
    command.args(args);
    command.creation_flags(CREATE_NO_WINDOW);
    match command.status() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn create_shortcut(target: &Path, shortcut_path: &Path, icon_path: &Path, desc: &str) -> bool {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let link_res: windows::core::Result<IShellLinkW> =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER);
        let link = match link_res {
            Ok(l) => l,
            Err(e) => {
                eprintln!("CoCreateInstance(ShellLink) failed: {}", e);
                return false;
            }
        };

        let target_w = to_wide(&target.display().to_string());
        let _ = link.SetPath(PCWSTR(target_w.as_ptr()));

        if let Some(parent) = target.parent() {
            let work_w = to_wide(&parent.display().to_string());
            let _ = link.SetWorkingDirectory(PCWSTR(work_w.as_ptr()));
        }

        let desc_w = to_wide(desc);
        let _ = link.SetDescription(PCWSTR(desc_w.as_ptr()));

        let icon_w = to_wide(&icon_path.display().to_string());
        let _ = link.SetIconLocation(PCWSTR(icon_w.as_ptr()), 0);

        let persist_file: windows::core::Result<IPersistFile> = link.cast();
        if let Ok(pf) = persist_file {
            let shortcut_w = to_wide(&shortcut_path.display().to_string());
            pf.Save(PCWSTR(shortcut_w.as_ptr()), true).is_ok()
        } else {
            false
        }
    }
}

fn reg_add(root: &str, key: &str, val_name: Option<&str>, val_data: &str) {
    let full_key = format!("{}\\{}", root, key);
    let mut args = vec!["add", &full_key];
    if let Some(name) = val_name {
        args.push("/v");
        args.push(name);
    } else {
        args.push("/ve");
    }
    args.push("/d");
    args.push(val_data);
    args.push("/f");
    run_command_hidden("reg.exe", &args);
}

fn reg_add_dword(root: &str, key: &str, val_name: &str, val_data: u32) {
    let full_key = format!("{}\\{}", root, key);
    let data_str = val_data.to_string();
    let args = [
        "add",
        &full_key,
        "/v",
        val_name,
        "/t",
        "REG_DWORD",
        "/d",
        &data_str,
        "/f",
    ];
    run_command_hidden("reg.exe", &args);
}

fn reg_delete(root: &str, key: &str) {
    let full_key = format!("{}\\{}", root, key);
    let args = ["delete", &full_key, "/f"];
    run_command_hidden("reg.exe", &args);
}

fn is_webview2_installed() -> bool {
    let mut check_cmd = Command::new("reg.exe");
    check_cmd.args([
        "query",
        "HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-9E47-44E9-B977-9E4935400D2B}",
        "/v",
        "pv",
    ]);
    check_cmd.creation_flags(CREATE_NO_WINDOW);
    if let Ok(res) = check_cmd.output() {
        if res.status.success() {
            return true;
        }
    }

    let mut check_user = Command::new("reg.exe");
    check_user.args([
        "query",
        "HKCU\\SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-9E47-44E9-B977-9E4935400D2B}",
        "/v",
        "pv",
    ]);
    check_user.creation_flags(CREATE_NO_WINDOW);
    if let Ok(res) = check_user.output() {
        if res.status.success() {
            return true;
        }
    }

    Path::new("C:\\Program Files (x86)\\Microsoft\\EdgeWebView\\Application").exists()
}

fn install_engine(
    target_dir: &Path,
    create_desktop: bool,
    create_start: bool,
    progress_callback: &mut dyn FnMut(i32, &str),
) -> Result<(), String> {
    progress_callback(5, "Creating destination directories...");
    if let Err(e) = fs::create_dir_all(target_dir) {
        return Err(format!("Failed to create directory {:?}: {}", target_dir, e));
    }

    let total = EMBEDDED_FILES.len();
    for (i, file) in EMBEDDED_FILES.iter().enumerate() {
        let dest_path = target_dir.join(file.path);
        if let Some(parent) = dest_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let msg = format!("Extracting {}...", file.path);
        let pct = 10 + ((i as f32 / total as f32) * 60.0) as i32;
        progress_callback(pct, &msg);

        if let Err(e) = fs::write(&dest_path, file.bytes) {
            return Err(format!("Failed to write {:?}: {}", dest_path, e));
        }
    }

    // Copy installer as uninstaller
    progress_callback(75, "Setting up uninstaller...");
    if let Ok(current_exe) = env::current_exe() {
        let uninstaller_dest = target_dir.join("uninstall.exe");
        let _ = fs::copy(&current_exe, &uninstaller_dest);
    }

    let main_exe = target_dir.join("Nexora.exe");
    let icon_ico = target_dir.join("assets").join("branding").join("icon.ico");

    // Create shortcuts
    if create_desktop {
        progress_callback(80, "Creating Desktop shortcut...");
        if let Some(desktop_lnk) = get_desktop_shortcut_path() {
            create_shortcut(&main_exe, &desktop_lnk, &icon_ico, "Nexora Desktop Browser");
        }
    }

    if create_start {
        progress_callback(85, "Creating Start Menu shortcut...");
        if let Some(start_lnk) = get_start_menu_shortcut_path() {
            create_shortcut(&main_exe, &start_lnk, &icon_ico, "Nexora Desktop Browser");
        }
    }

    // Write Registry entries
    progress_callback(90, "Registering Windows integration...");
    let main_exe_str = main_exe.display().to_string();
    let target_dir_str = target_dir.display().to_string();
    let uninst_str = format!("\"{}\"", target_dir.join("uninstall.exe").display());
    let icon_ref = format!("{},0", main_exe_str);

    // App Paths
    let app_paths = "Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\Nexora.exe";
    reg_add("HKCU", app_paths, None, &main_exe_str);
    reg_add("HKCU", app_paths, Some("Path"), &target_dir_str);

    // Uninstall
    let uninst_key = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Nexora";
    reg_add("HKCU", uninst_key, Some("DisplayName"), "Nexora");
    reg_add("HKCU", uninst_key, Some("DisplayVersion"), "0.1.0");
    reg_add("HKCU", uninst_key, Some("Publisher"), "Nexora Development Team");
    reg_add("HKCU", uninst_key, Some("DisplayIcon"), &icon_ref);
    reg_add("HKCU", uninst_key, Some("UninstallString"), &uninst_str);
    reg_add(
        "HKCU",
        uninst_key,
        Some("QuietUninstallString"),
        &format!("{} /S", uninst_str),
    );
    reg_add("HKCU", uninst_key, Some("InstallLocation"), &target_dir_str);
    reg_add("HKCU", uninst_key, Some("HelpLink"), "https://github.com/Nexora/Nexora");
    reg_add("HKCU", uninst_key, Some("URLInfoAbout"), "https://github.com/Nexora/Nexora");
    reg_add_dword("HKCU", uninst_key, "EstimatedSize", 4200);
    reg_add_dword("HKCU", uninst_key, "NoModify", 1);
    reg_add_dword("HKCU", uninst_key, "NoRepair", 1);

    // URL Protocol
    reg_add("HKCU", "Software\\Classes\\nexora", None, "URL:Nexora Protocol");
    reg_add("HKCU", "Software\\Classes\\nexora", Some("URL Protocol"), "");
    reg_add("HKCU", "Software\\Classes\\nexora\\DefaultIcon", None, &icon_ref);
    reg_add(
        "HKCU",
        "Software\\Classes\\nexora\\shell\\open\\command",
        None,
        &format!("\"{}\" \"%1\"", main_exe_str),
    );

    progress_callback(100, "Installation complete!");
    Ok(())
}

fn run_uninstaller(silent: bool) {
    if !silent {
        let title = to_wide("Nexora Uninstall");
        let msg = to_wide("Are you sure you want to completely remove Nexora from your computer?");
        unsafe {
            let res = MessageBoxW(
                HWND(std::ptr::null_mut()),
                PCWSTR(msg.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_YESNO | MB_ICONQUESTION,
            );
            if res.0 != 6 {
                // IDYES = 6
                return;
            }
        }
    }

    // 1. Terminate running Nexora processes
    run_command_hidden("cmd.exe", &["/C", "taskkill", "/F", "/IM", "Nexora.exe", "/T"]);

    // 2. Remove Shortcuts
    if let Some(desktop_lnk) = get_desktop_shortcut_path() {
        let _ = fs::remove_file(desktop_lnk);
    }
    if let Some(start_lnk) = get_start_menu_shortcut_path() {
        let _ = fs::remove_file(start_lnk);
    }

    // 3. Remove Registry Keys
    reg_delete("HKCU", "Software\\Microsoft\\Windows\\CurrentVersion\\App Paths\\Nexora.exe");
    reg_delete("HKCU", "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Nexora");
    reg_delete("HKCU", "Software\\Classes\\nexora");

    // 4. Schedule directory removal asynchronously so this uninstaller process exits and unlocks the folder
    let install_dir = if let Ok(current_exe) = env::current_exe() {
        current_exe.parent().map(|p| p.to_path_buf()).unwrap_or_else(get_default_install_dir)
    } else {
        get_default_install_dir()
    };

    let dir_str = install_dir.display().to_string().replace("'", "''");
    let script = format!("Start-Sleep -Seconds 2; Remove-Item -Recurse -Force '{}'", dir_str);
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script]);
    cmd.creation_flags(CREATE_NO_WINDOW);
    let _ = cmd.spawn();

    if !silent {
        let title = to_wide("Nexora Uninstall");
        let msg = to_wide("Nexora was successfully uninstalled from your computer.");
        unsafe {
            MessageBoxW(
                HWND(std::ptr::null_mut()),
                PCWSTR(msg.as_ptr()),
                PCWSTR(title.as_ptr()),
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }
}

// Win32 GUI State
struct GuiState {
    h_brush_bg: HBRUSH,
    h_brush_card: HBRUSH,
    h_brush_btn_crimson: HBRUSH,
    h_brush_btn_crimson_hover: HBRUSH,
    h_brush_btn_dark: HBRUSH,
    h_font_title: isize,
    h_font_sub: isize,
    h_font_body: isize,
    h_font_bold: isize,
    h_icon: HICON,
    chk_desktop: HWND,
    chk_start: HWND,
    chk_launch: HWND,
    progress: HWND,
    status_label: HWND,
    btn_install: HWND,
    btn_cancel: HWND,
}

static mut GLOBAL_STATE: Option<GuiState> = None;

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if let Some(ref state) = GLOBAL_STATE {
                // Background
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                FillRect(hdc, &rect, state.h_brush_bg);

                // Header Banner Icon
                if !state.h_icon.is_invalid() {
                    let _ = DrawIconEx(hdc, 24, 20, state.h_icon, 48, 48, 0, HBRUSH(std::ptr::null_mut()), DI_NORMAL);
                }

                // Title Text
                SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(state.h_font_title as *mut _));
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, COLORREF(0x00FAFAFA)); // Soft white
                let title_text = to_wide("Nexora");
                let _ = TextOutW(hdc, 84, 18, &title_text[..title_text.len() - 1]);

                // Version Badge Text
                SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(state.h_font_sub as *mut _));
                SetTextColor(hdc, COLORREF(0x00481DE1)); // Crimson accent in BGR
                let ver_text = to_wide("v0.1.0");
                let _ = TextOutW(hdc, 195, 23, &ver_text[..ver_text.len() - 1]);

                // Subtitle
                SetTextColor(hdc, COLORREF(0x00A1A1AA)); // Muted grey
                let sub_text = to_wide("Ultra-Lightweight, Streaming-First Desktop Browser");
                let _ = TextOutW(hdc, 84, 46, &sub_text[..sub_text.len() - 1]);

                // Separator Line
                let sep_pen = CreatePen(PS_SOLID, 1, COLORREF(0x002A2727));
                let old_pen = SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(sep_pen.0 as *mut _));
                let _ = MoveToEx(hdc, 24, 82, None);
                let _ = LineTo(hdc, 496, 82);
                SelectObject(hdc, old_pen);
                let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(sep_pen.0 as *mut _));

                // Bottom Separator Line
                let sep_pen2 = CreatePen(PS_SOLID, 1, COLORREF(0x002A2727));
                let old_pen2 = SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(sep_pen2.0 as *mut _));
                let _ = MoveToEx(hdc, 24, 320, None);
                let _ = LineTo(hdc, 496, 320);
                SelectObject(hdc, old_pen2);
                let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(sep_pen2.0 as *mut _));

                // "Destination Folder:" static text
                SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(state.h_font_bold as *mut _));
                SetTextColor(hdc, COLORREF(0x00D4D4D8));
                let dest_lbl = to_wide("Destination Folder:");
                let _ = TextOutW(hdc, 24, 98, &dest_lbl[..dest_lbl.len() - 1]);
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_CTLCOLORSTATIC => {
            let hdc = HDC(wparam.0 as *mut _);
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLORREF(0x00D4D4D8));
            if let Some(ref state) = GLOBAL_STATE {
                LRESULT(state.h_brush_bg.0 as isize)
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }

        WM_CTLCOLOREDIT => {
            let hdc = HDC(wparam.0 as *mut _);
            SetBkColor(hdc, COLORREF(0x001B1818));
            SetTextColor(hdc, COLORREF(0x00FAFAFA));
            if let Some(ref state) = GLOBAL_STATE {
                LRESULT(state.h_brush_card.0 as isize)
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }

        WM_CTLCOLORBTN => {
            if let Some(ref state) = GLOBAL_STATE {
                LRESULT(state.h_brush_bg.0 as isize)
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }

        WM_DRAWITEM => {
            let dis = &*(lparam.0 as *const DRAWITEMSTRUCT);
            let hdc = dis.hDC;
            let is_pressed = (dis.itemState.0 & ODS_SELECTED.0) != 0;

            if let Some(ref state) = GLOBAL_STATE {
                if dis.CtlID as usize == ID_BTN_INSTALL {
                    let brush = if is_pressed {
                        state.h_brush_btn_crimson_hover
                    } else {
                        state.h_brush_btn_crimson
                    };
                    SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(brush.0 as *mut _));
                    let null_pen = GetStockObject(NULL_PEN);
                    SelectObject(hdc, null_pen);
                    let _ = RoundRect(hdc, dis.rcItem.left, dis.rcItem.top, dis.rcItem.right, dis.rcItem.bottom, 6, 6);

                    SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(state.h_font_bold as *mut _));
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, COLORREF(0x00FFFFFF));

                    let mut label = if IS_FINISHED.load(Ordering::SeqCst) {
                        to_wide("Finish")
                    } else {
                        to_wide("Install Nexora")
                    };
                    let len = label.len().saturating_sub(1);
                    let mut text_rect = dis.rcItem;
                    let _ = DrawTextW(
                        hdc,
                        &mut label[..len],
                        &mut text_rect,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                    return LRESULT(1);
                } else if dis.CtlID as usize == ID_BTN_CANCEL {
                    SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(state.h_brush_btn_dark.0 as *mut _));
                    let border_pen = CreatePen(PS_SOLID, 1, COLORREF(0x00463F3F));
                    SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(border_pen.0 as *mut _));
                    let _ = RoundRect(hdc, dis.rcItem.left, dis.rcItem.top, dis.rcItem.right, dis.rcItem.bottom, 6, 6);
                    let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(border_pen.0 as *mut _));

                    SelectObject(hdc, windows::Win32::Graphics::Gdi::HGDIOBJ(state.h_font_body as *mut _));
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, COLORREF(0x00D4D4D8));
                    let mut label = to_wide("Cancel");
                    let len = label.len().saturating_sub(1);
                    let mut text_rect = dis.rcItem;
                    let _ = DrawTextW(
                        hdc,
                        &mut label[..len],
                        &mut text_rect,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                    return LRESULT(1);
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as usize;
            if id == ID_BTN_CANCEL {
                let _ = DestroyWindow(hwnd);
            } else if id == ID_BTN_INSTALL {
                if IS_FINISHED.load(Ordering::SeqCst) {
                    // Finished: launch if requested and exit
                    if let Some(ref state) = GLOBAL_STATE {
                        let launch = SendMessageW(state.chk_launch, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == BST_CHECKED_VAL as isize;
                        if launch {
                            let main_exe = get_default_install_dir().join("Nexora.exe");
                            let _ = Command::new(&main_exe).spawn();
                        }
                    }
                    let _ = DestroyWindow(hwnd);
                    return LRESULT(0);
                }

                // Check WebView2
                if !is_webview2_installed() {
                    let title = to_wide("WebView2 Runtime Required");
                    let msg = to_wide(
                        "Nexora requires the Microsoft Edge WebView2 Evergreen Runtime.\n\n\
                         Would you like to open Microsoft's download page now?",
                    );
                    let res = MessageBoxW(
                        hwnd,
                        PCWSTR(msg.as_ptr()),
                        PCWSTR(title.as_ptr()),
                        MB_YESNO | MB_ICONQUESTION,
                    );
                    if res.0 == 6 {
                        // IDYES
                        let _ = Command::new("cmd.exe")
                            .args(["/C", "start", "https://go.microsoft.com/fwlink/p/?LinkId=2124703"])
                            .spawn();
                    }
                }

                if let Some(ref state) = GLOBAL_STATE {
                    let _ = EnableWindow(state.btn_install, BOOL::from(false));
                    let _ = EnableWindow(state.btn_cancel, BOOL::from(false));

                    let create_desktop = SendMessageW(state.chk_desktop, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == BST_CHECKED_VAL as isize;
                    let create_start = SendMessageW(state.chk_start, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == BST_CHECKED_VAL as isize;

                    let target_dir = get_default_install_dir();
                    let progress_hwnd = state.progress;
                    let status_hwnd = state.status_label;

                    let mut callback = |pct: i32, text: &str| {
                        SendMessageW(progress_hwnd, PBM_SETPOS, WPARAM(pct as usize), LPARAM(0));
                        let wide_text = to_wide(text);
                        let _ = SetWindowTextW(status_hwnd, PCWSTR(wide_text.as_ptr()));

                        // Pump messages to redraw UI
                        let mut msg = MSG::default();
                        while PeekMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0, PM_REMOVE).as_bool() {
                            let _ = TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    };

                    match install_engine(&target_dir, create_desktop, create_start, &mut callback) {
                        Ok(()) => {
                            IS_FINISHED.store(true, Ordering::SeqCst);
                            let _ = EnableWindow(state.btn_install, BOOL::from(true));
                            let _ = SetWindowTextW(state.btn_install, PCWSTR(to_wide("Finish").as_ptr()));
                            let _ = InvalidateRect(state.btn_install, None, true);

                            let done_msg = to_wide("Nexora has been installed successfully!");
                            let _ = SetWindowTextW(state.status_label, PCWSTR(done_msg.as_ptr()));
                        }
                        Err(e) => {
                            let title = to_wide("Installation Error");
                            let err_wide = to_wide(&format!("Installation failed: {}", e));
                            MessageBoxW(hwnd, PCWSTR(err_wide.as_ptr()), PCWSTR(title.as_ptr()), MB_OK | MB_ICONERROR);
                            let _ = EnableWindow(state.btn_install, BOOL::from(true));
                            let _ = EnableWindow(state.btn_cancel, BOOL::from(true));
                        }
                    }
                }
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn run_installer_gui() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let mut icex = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: INITCOMMONCONTROLSEX_ICC(0x00000020), // ICC_PROGRESS_CLASS
        };
        let _ = InitCommonControlsEx(&mut icex);

        let h_instance = GetModuleHandleW(PCWSTR(std::ptr::null())).unwrap_or_default();
        let class_name = to_wide("NexoraSetupWindowClass");

        let h_brush_bg = CreateSolidBrush(COLORREF(0x001B1818)); // #18181B in BGR (0x1B, 0x18, 0x18)
        let h_brush_card = CreateSolidBrush(COLORREF(0x00272020)); // #202027 in BGR
        let h_brush_btn_crimson = CreateSolidBrush(COLORREF(0x00481DE1)); // #E11D48 in BGR (0x48, 0x1D, 0xE1)
        let h_brush_btn_crimson_hover = CreateSolidBrush(COLORREF(0x003C12BE)); // #BE123C in BGR
        let h_brush_btn_dark = CreateSolidBrush(COLORREF(0x002A2727)); // #27272A in BGR

        let font_title = CreateFontW(
            -22, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 0, 0, 0, 5, 0, PCWSTR(to_wide("Segoe UI").as_ptr()),
        );
        let font_sub = CreateFontW(
            -13, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0, 0, 0, 0, 5, 0, PCWSTR(to_wide("Segoe UI").as_ptr()),
        );
        let font_body = CreateFontW(
            -12, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0, 0, 0, 0, 5, 0, PCWSTR(to_wide("Segoe UI").as_ptr()),
        );
        let font_bold = CreateFontW(
            -12, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0, 0, 0, 0, 5, 0, PCWSTR(to_wide("Segoe UI").as_ptr()),
        );

        let h_icon = LoadIconW(h_instance, PCWSTR(1 as *const u16)).unwrap_or_else(|_| {
            LoadIconW(HINSTANCE(std::ptr::null_mut()), IDI_APPLICATION).unwrap_or_default()
        });

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance.into(),
            hIcon: h_icon,
            hCursor: HCURSOR(std::ptr::null_mut()),
            hbrBackground: h_brush_bg,
            lpszMenuName: PCWSTR(std::ptr::null()),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hIconSm: h_icon,
        };

        RegisterClassExW(&wc);

        let win_width = 520;
        let win_height = 430;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let x = (screen_w - win_width) / 2;
        let y = (screen_h - win_height) / 2;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(to_wide("Nexora Setup").as_ptr()),
            WINDOW_STYLE(WS_OVERLAPPED.0 | WS_CAPTION | WS_SYSMENU.0 | WS_MINIMIZEBOX.0 | WS_CLIPCHILDREN.0),
            x,
            y,
            win_width,
            win_height,
            HWND(std::ptr::null_mut()),
            windows::Win32::UI::WindowsAndMessaging::HMENU(std::ptr::null_mut()),
            h_instance,
            None,
        ).unwrap();

        // Path Edit Control (Readonly)
        let default_dir = get_default_install_dir();
        let default_dir_wide = to_wide(&default_dir.display().to_string());
        let edit_path = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(to_wide("EDIT").as_ptr()),
            PCWSTR(default_dir_wide.as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32 | ES_READONLY as u32),
            24,
            122,
            472,
            26,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_PATH_EDIT as *mut _),
            h_instance,
            None,
        ).unwrap();
        SendMessageW(edit_path, windows::Win32::UI::WindowsAndMessaging::WM_SETFONT, WPARAM(font_body.0 as usize), LPARAM(1));

        // Checkbox: Desktop Shortcut
        let chk_desktop = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(to_wide("BUTTON").as_ptr()),
            PCWSTR(to_wide("Create Desktop shortcut").as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            24,
            158,
            400,
            22,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_CHK_DESKTOP as *mut _),
            h_instance,
            None,
        ).unwrap();
        SendMessageW(chk_desktop, BM_SETCHECK, WPARAM(BST_CHECKED_VAL), LPARAM(0));
        SendMessageW(chk_desktop, windows::Win32::UI::WindowsAndMessaging::WM_SETFONT, WPARAM(font_body.0 as usize), LPARAM(1));

        // Checkbox: Start Menu Shortcut
        let chk_start = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(to_wide("BUTTON").as_ptr()),
            PCWSTR(to_wide("Create Start Menu shortcut").as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            24,
            184,
            400,
            22,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_CHK_START as *mut _),
            h_instance,
            None,
        ).unwrap();
        SendMessageW(chk_start, BM_SETCHECK, WPARAM(BST_CHECKED_VAL), LPARAM(0));
        SendMessageW(chk_start, windows::Win32::UI::WindowsAndMessaging::WM_SETFONT, WPARAM(font_body.0 as usize), LPARAM(1));

        // Checkbox: Launch Nexora
        let chk_launch = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(to_wide("BUTTON").as_ptr()),
            PCWSTR(to_wide("Launch Nexora when setup completes").as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_AUTOCHECKBOX as u32),
            24,
            210,
            400,
            22,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_CHK_LAUNCH as *mut _),
            h_instance,
            None,
        ).unwrap();
        SendMessageW(chk_launch, BM_SETCHECK, WPARAM(BST_CHECKED_VAL), LPARAM(0));
        SendMessageW(chk_launch, windows::Win32::UI::WindowsAndMessaging::WM_SETFONT, WPARAM(font_body.0 as usize), LPARAM(1));

        // Progress Bar
        let progress = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PROGRESS_CLASS,
            PCWSTR(std::ptr::null()),
            WS_CHILD | WS_VISIBLE,
            24,
            248,
            472,
            14,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_PROGRESS as *mut _),
            h_instance,
            None,
        ).unwrap();
        SendMessageW(progress, PBM_SETRANGE32, WPARAM(0), LPARAM(100));
        SendMessageW(progress, PBM_SETPOS, WPARAM(0), LPARAM(0));

        // Status Label
        let status_label = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(to_wide("STATIC").as_ptr()),
            PCWSTR(to_wide("Ready to install Nexora v0.1.0").as_ptr()),
            WS_CHILD | WS_VISIBLE,
            24,
            272,
            472,
            20,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_STATUS as *mut _),
            h_instance,
            None,
        ).unwrap();
        SendMessageW(status_label, windows::Win32::UI::WindowsAndMessaging::WM_SETFONT, WPARAM(font_body.0 as usize), LPARAM(1));

        // Install Button (Ownerdrawn Crimson)
        let btn_install = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(to_wide("BUTTON").as_ptr()),
            PCWSTR(to_wide("Install Nexora").as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
            290,
            340,
            124,
            36,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_BTN_INSTALL as *mut _),
            h_instance,
            None,
        ).unwrap();

        // Cancel Button (Ownerdrawn Dark)
        let btn_cancel = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(to_wide("BUTTON").as_ptr()),
            PCWSTR(to_wide("Cancel").as_ptr()),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
            424,
            340,
            72,
            36,
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::HMENU(ID_BTN_CANCEL as *mut _),
            h_instance,
            None,
        ).unwrap();

        GLOBAL_STATE = Some(GuiState {
            h_brush_bg,
            h_brush_card,
            h_brush_btn_crimson,
            h_brush_btn_crimson_hover,
            h_brush_btn_dark,
            h_font_title: font_title.0 as isize,
            h_font_sub: font_sub.0 as isize,
            h_font_body: font_body.0 as isize,
            h_font_bold: font_bold.0 as isize,
            h_icon,
            chk_desktop,
            chk_start,
            chk_launch,
            progress,
            status_label,
            btn_install,
            btn_cancel,
        });

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup GDI objects
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(h_brush_bg.0 as *mut _));
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(h_brush_card.0 as *mut _));
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(h_brush_btn_crimson.0 as *mut _));
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(h_brush_btn_crimson_hover.0 as *mut _));
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(h_brush_btn_dark.0 as *mut _));
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(font_title.0 as *mut _));
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(font_sub.0 as *mut _));
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(font_body.0 as *mut _));
        let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(font_bold.0 as *mut _));
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let current_exe_name = env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
        .unwrap_or_default();

    let is_uninstall = current_exe_name.contains("uninstall")
        || args.iter().any(|a| a == "--uninstall" || a == "/uninstall");

    let is_silent = args.iter().any(|a| a == "/S" || a == "-s" || a == "--silent" || a == "/silent");

    if is_uninstall {
        run_uninstaller(is_silent);
        return;
    }

    if is_silent {
        let target_dir = get_default_install_dir();
        let mut silent_cb = |_pct: i32, text: &str| {
            println!("{}", text);
        };
        if let Err(e) = install_engine(&target_dir, true, true, &mut silent_cb) {
            eprintln!("Installation error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    run_installer_gui();
}
