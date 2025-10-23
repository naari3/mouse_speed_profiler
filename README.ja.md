# MouseSpeedProfiler

MouseSpeedProfilerは、フォーカス中のアプリが切り替わったタイミングで自動的にOSのマウスポインターの速度を調整するツールです。主に、MinecraftのRTAで使用されるテクニックである **ボート測量** のために作られました。

[English](README.md) / **日本語**

## 特徴

- **自動的な設定変更**: アプリケーション間の切り替え時に自動的にOSのマウスポインターの速度を変更します。
- **設定可能なルール**: 設定ファイルを使用して、特定のアプリケーションに対して個別のマウス速度を設定できます。

## 設定

初回実行時に `config.toml` テンプレートが作成されます。このファイルを編集して、アプリケーション個別のマウス速度を設定してください。

### 設定ファイル

```toml
[[rules]]
window_title = "Minecraft"
exe_name = "javaw.exe"
match_all = true
speed = 5

[[rules]]
window_title = "Minecraft"
exe_name = "java.exe"
match_all = true
speed = 5

default_speed = 10
```

- **`window_title`**: アプリケーションウィンドウのタイトル（オプション）
  - ウィンドウタイトルと前方一致したものを対象とします
- **`exe_name`**: アプリケーションの実行ファイル名（オプション）
  - 実行ファイルのパスと後方一致したものを対象とします
- **`match_all`**: 両方の条件を満たす必要があるかどうか（オプション、デフォルト: true）
  - `true`: `window_title`と`exe_name`の両方が一致する必要があります（両方が指定されている場合）
  - `false`: `window_title`または`exe_name`のいずれかが一致すればよい
- **`speed`**: アプリケーションの望ましいマウス速度（1-20）
