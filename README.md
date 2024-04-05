# MouseSpeedProfiler

MouseSpeedProfiler is a tool that automatically adjusts the OS mouse pointer speed when the focus shifts between applications. It was primarily created for the technique known as **Boat measurements** used in Minecraft Speedrun.

**English** / [日本語](README.ja.md)

## Features

- **Automatic Setting Change**: Automatically changes the OS mouse pointer speed when switching between applications.
- **Configurable Rules**: Use a configuration file to set individual mouse speeds for specific applications.

## Configuration

Upon the first run, a `config.toml` template will be created. Edit this file to set application-specific mouse speeds.

### Configuration File

```toml
[[rules]]
window_title = "Minecraft"
exe_name = "javaw.exe"
speed = 5

[[rules]]
window_title = "Minecraft"
exe_name = "java.exe"
speed = 5

default_speed = 10
```

- **`window_title`**: The title of the application window
  - Targets are determined by a prefix match with the window title
- **`exe_name`**: The executable file name of the application (optional)
  - If specified, targets are further filtered by matching the executable file name in addition to the title
- **`speed`**: The desired mouse speed for the application (1-20)
