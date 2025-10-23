use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::path::PathBuf;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::Threading::*,
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
};

static RECONCILER: OnceLock<Reconciler> = OnceLock::new();

fn get_config_path() -> Result<PathBuf> {
    let app_data = std::env::var("APPDATA").map_err(|_| Error::from_win32())?;
    let mut path = PathBuf::from(app_data);
    path.push("MouseSpeedProfiler");
    
    // Create directory if it doesn't exist
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|_| Error::from_win32())?;
    }
    
    path.push("config.toml");
    Ok(path)
}

fn create_config_template() -> Result<()> {
    let config = Config {
        rules: vec![
            Rule {
                window_title: Some("Minecraft".to_string()),
                exe_name: Some("javaw.exe".to_string()),
                match_all: true,
                speed: 5,
            },
            Rule {
                window_title: Some("Minecraft".to_string()),
                exe_name: Some("java.exe".to_string()),
                match_all: true,
                speed: 5,
            },
        ],
        default_speed: 10,
    };
    let config = toml::to_string_pretty(&config).expect("Failed to serialize config");
    let config_path = get_config_path()?;
    std::fs::write(&config_path, config).expect("Failed to write config file");
    error!("Config template created at {}", config_path.display());
    Ok(())
}

fn init_reconciler() -> Reconciler {
    let config_path = get_config_path().expect("Failed to get config path");
    let config = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to read config file: {}", e);
            error!("Creating config template...");
            create_config_template().expect("Failed to create config template");
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
    window_title: Option<String>,
    exe_name: Option<String>,
    #[serde(default = "default_match_all")]
    match_all: bool,
    speed: usize,
}

fn default_match_all() -> bool {
    true
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
            trace!("Matching rule: {:?}", rule);
            let matched_title = rule
                .window_title
                .as_ref()
                .is_some_and(|t| title.starts_with(t));

            let matched_exe = rule
                .exe_name
                .as_ref()
                .is_some_and(|e| exe_path.ends_with(e));

            let matched = if rule.match_all {
                matched_title && matched_exe
            } else {
                matched_title || matched_exe
            };

            if matched {
                trace!("Matched: {:?}", rule.speed);
                return rule.speed;
            }
        }
        trace!(
            "No match, using default speed: {}",
            self.config.default_speed
        );
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
