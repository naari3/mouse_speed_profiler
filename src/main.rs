use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::Threading::*,
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
};

static RECONCILER: OnceLock<Reconciler> = OnceLock::new();

fn create_config_template() -> Result<()> {
    let config = Config {
        rules: vec![
            Rule {
                window_title: "Minecraft".to_string(),
                exe_name: Some("javaw.exe".to_string()),
                speed: 5,
            },
            Rule {
                window_title: "Minecraft".to_string(),
                exe_name: Some("java.exe".to_string()),
                speed: 5,
            },
        ],
        default_speed: 10,
    };
    let config = toml::to_string_pretty(&config).expect("Failed to serialize config");
    std::fs::write("config.toml", config).expect("Failed to write config file");
    Ok(())
}

fn init_reconciler() -> Reconciler {
    let config = match std::fs::read_to_string("config.toml") {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to read config file: {}", e);
            error!("Creating config template...");
            create_config_template().expect("Failed to create config template");
            error!("Config template created at config.toml");
            error!("Please edit the config file and restart the program");
            error!("Press Enter key to exit...");
            let _ = std::io::stdin().read_line(&mut String::new());
            std::process::exit(1);
        }
    };
    let config: Config = toml::from_str(&config).expect("Failed to parse config file");
    Reconciler::new(config)
}

#[derive(Debug, Serialize, Deserialize)]
struct Rule {
    window_title: String,
    exe_name: Option<String>,
    speed: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    rules: Vec<Rule>,
    default_speed: usize,
}

#[derive(Debug)]
struct Reconciler {
    config: Config,
}

impl Reconciler {
    fn new(config: Config) -> Self {
        Self { config }
    }

    fn reconcile(&self, title: &str, exe_path: &str) -> Result<()> {
        let speed = self.get_reconciled_speed(title, exe_path);
        debug!("Reconciling {} with speed {}", title, speed);
        let current_speed = self.get_speed()?;
        if speed == current_speed {
            debug!("Speed already set to {}", speed);
            return Ok(());
        }
        self.set_speed(speed)?;
        info!("Set speed to {}", speed);
        Ok(())
    }

    fn get_reconciled_speed(&self, title: &str, exe_path: &str) -> usize {
        for rule in &self.config.rules {
            if title.starts_with(&rule.window_title) {
                if let Some(exe_name) = &rule.exe_name {
                    if exe_path.ends_with(exe_name) {
                        return rule.speed;
                    }
                } else {
                    return rule.speed;
                }
            }
        }
        self.config.default_speed
    }

    fn set_speed(&self, speed: usize) -> Result<()> {
        unsafe {
            SystemParametersInfoW(SPI_SETMOUSESPEED, 0, Some(speed as _), SPIF_SENDCHANGE)?;
        }
        Ok(())
    }

    fn get_speed(&self) -> Result<usize> {
        let mut speed: i32 = 0;
        unsafe {
            SystemParametersInfoW(
                SPI_GETMOUSESPEED,
                0,
                Some(&mut speed as *mut i32 as *mut std::ffi::c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )?;
        }
        Ok(speed as usize)
    }
}

unsafe extern "system" fn event_callback(
    _h_win_event_hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    let title = get_window_title(hwnd).unwrap_or_else(|_| "Unknown".to_string());
    let _pid = get_window_pid(hwnd).unwrap_or(0);
    let exe_path = get_window_exe_path(hwnd).unwrap_or_else(|_| "Unknown".to_string());

    RECONCILER
        .get()
        .unwrap()
        .reconcile(&title, &exe_path)
        .unwrap();
}

unsafe fn get_window_title(hwnd: HWND) -> Result<String> {
    let length = GetWindowTextLengthW(hwnd) + 1; // +1 for null terminator
    let mut title = vec![0u16; length as usize];
    GetWindowTextW(hwnd, &mut title);

    // Convert to String and trim null terminator if present
    if let Some(end) = title.iter().position(|&c| c == 0) {
        title.truncate(end);
    }
    Ok(String::from_utf16(&title).expect("Failed to decode UTF-16"))
}

unsafe fn get_window_pid(hwnd: HWND) -> Result<u32> {
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    Ok(pid)
}

unsafe fn get_window_exe_path(hwnd: HWND) -> Result<String> {
    let pid = get_window_pid(hwnd)?;

    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)?;
    let mut buffer = [0u16; 4096];
    let mut size = buffer.len() as u32;
    QueryFullProcessImageNameW(
        handle,
        PROCESS_NAME_WIN32,
        PWSTR(buffer.as_mut_ptr()),
        &mut size,
    )?;
    CloseHandle(handle)?;

    // Convert to String and trim null terminator if present
    if let Some(end) = buffer.iter().position(|&c| c == 0) {
        buffer[end] = 0;
    }
    Ok(String::from_utf16(&buffer[..size as usize]).expect("Failed to decode UTF-16"))
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let _ = RECONCILER.get_or_init(init_reconciler);
    unsafe {
        let event_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        let mut msg: MSG = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWinEvent(event_hook);
    }
    Ok(())
}
