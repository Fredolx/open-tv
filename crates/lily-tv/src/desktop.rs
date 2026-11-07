//! Windows desktop integration: one running instance, and a tray icon the window
//! can hide into. Other platforms run without either.

#[cfg(windows)]
pub use windows_impl::*;

#[cfg(not(windows))]
mod fallback {
    use gpui_kit::{App, Window};

    pub struct SingleInstance;

    impl SingleInstance {
        pub fn claim() -> Option<Self> {
            Some(Self)
        }

        pub fn on_activate(self, _: impl Fn(&mut App) + 'static, _: &mut App) {}
    }

    pub fn install_tray(_: bool, _: &mut Window, _: &mut App) {}

    pub fn window_raw(_: &Window) -> Option<isize> {
        None
    }

    pub fn show_window(_: isize) {}
}

#[cfg(not(windows))]
pub use fallback::*;

#[cfg(windows)]
mod windows_impl {
    use std::{
        cell::Cell,
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
        rc::Rc,
    };

    use futures::{StreamExt as _, channel::mpsc};
    use gpui_kit::{App, Global, Window};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use tray_icon::{
        Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
        menu::{Menu, MenuEvent, MenuItem},
    };
    use windows_sys::Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND},
        System::Threading::{CreateEventW, CreateMutexW, INFINITE, SetEvent, WaitForSingleObject},
        UI::WindowsAndMessaging::{SW_HIDE, SW_RESTORE, SW_SHOW, SetForegroundWindow, ShowWindow},
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Names are per data folder, so a test profile runs beside the normal one.
    fn object_name(kind: &str) -> String {
        let mut hasher = DefaultHasher::new();
        lily_core::paths::data_dir().hash(&mut hasher);
        format!("Local\\LilyTV-{kind}-{:x}", hasher.finish())
    }

    /// Proof this is the only Lily TV for this data folder.
    pub struct SingleInstance {
        _mutex: HANDLE,
        activate_event: HANDLE,
    }

    impl SingleInstance {
        /// Returns None, after asking the running instance to show itself, if one exists.
        pub fn claim() -> Option<Self> {
            let mutex_name = wide(&object_name("instance"));
            let event_name = wide(&object_name("activate"));
            unsafe {
                let mutex = CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr());
                let already_running = GetLastError() == ERROR_ALREADY_EXISTS;
                // auto-reset event the running instance waits on
                let event = CreateEventW(std::ptr::null(), 0, 0, event_name.as_ptr());
                if already_running {
                    if !event.is_null() {
                        SetEvent(event);
                        CloseHandle(event);
                    }
                    CloseHandle(mutex);
                    return None;
                }
                Some(Self { _mutex: mutex, activate_event: event })
            }
        }

        /// Calls `activate` on the UI thread whenever another launch is attempted.
        pub fn on_activate(self, activate: impl Fn(&mut App) + 'static, cx: &mut App) {
            let (tx, mut rx) = mpsc::unbounded::<()>();
            // HANDLE is a raw pointer; pass it as an integer to the waiting thread
            let event = self.activate_event as usize;
            std::thread::Builder::new()
                .name("lily-single-instance".into())
                .spawn(move || loop {
                    if unsafe { WaitForSingleObject(event as HANDLE, INFINITE) } != 0 {
                        break;
                    }
                    if tx.unbounded_send(()).is_err() {
                        break;
                    }
                })
                .ok();
            cx.spawn(async move |cx| {
                while rx.next().await.is_some() {
                    cx.update(|cx| activate(cx));
                }
            })
            .detach();
            // the handles are never closed, so the mutex stays held for the life of the process
        }
    }

    struct Tray {
        _icon: TrayIcon,
    }

    impl Global for Tray {}

    fn hwnd(window: &Window) -> Option<HWND> {
        match HasWindowHandle::window_handle(window).ok()?.as_raw() {
            RawWindowHandle::Win32(handle) => Some(handle.hwnd.get() as HWND),
            _ => None,
        }
    }

    /// Brings a hidden or minimized window back to the front.
    pub fn show_window(raw: isize) {
        let hwnd = raw as HWND;
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
    }

    /// Adds the tray icon (Show / Quit). With `close_to_tray`, closing the window
    /// hides it instead of quitting.
    pub fn install_tray(close_to_tray: bool, window: &mut Window, cx: &mut App) {
        let Some(hwnd) = hwnd(window) else {
            return;
        };
        let raw = hwnd as isize;
        let show = MenuItem::new("Show Lily TV", true, None);
        let quit = MenuItem::new("Quit", true, None);
        let menu = Menu::new();
        if menu.append_items(&[&show, &quit]).is_err() {
            return;
        }
        let mut builder = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip("Lily TV");
        // the exe's embedded icon (winresource puts it at ordinal 1)
        if let Ok(icon) = Icon::from_resource(1, None) {
            builder = builder.with_icon(icon);
        }
        let Ok(icon) = builder.build() else {
            lily_core::log::log("could not create the tray icon".to_string());
            return;
        };
        cx.set_global(Tray { _icon: icon });

        enum TrayAction {
            Show,
            Quit,
        }
        let (tx, mut rx) = mpsc::unbounded::<TrayAction>();
        let (show_id, quit_id) = (show.id().clone(), quit.id().clone());
        let menu_tx = tx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = if event.id == show_id {
                TrayAction::Show
            } else if event.id == quit_id {
                TrayAction::Quit
            } else {
                return;
            };
            _ = menu_tx.unbounded_send(action);
        }));
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                _ = tx.unbounded_send(TrayAction::Show);
            }
        }));
        let quitting = Rc::new(Cell::new(false));
        let quitting_tray = quitting.clone();
        cx.spawn(async move |cx| {
            while let Some(action) = rx.next().await {
                match action {
                    TrayAction::Show => show_window(raw),
                    TrayAction::Quit => {
                        quitting_tray.set(true);
                        cx.update(|cx| cx.quit());
                    }
                }
            }
        })
        .detach();

        if close_to_tray {
            window.on_window_should_close(cx, move |_, _| {
                if quitting.get() {
                    return true;
                }
                unsafe { ShowWindow(raw as HWND, SW_HIDE) };
                false
            });
        }
    }

    /// Lets the window's owner bring it forward from another thread's request.
    pub fn window_raw(window: &Window) -> Option<isize> {
        hwnd(window).map(|h| h as isize)
    }
}
