$tempDir = New-TemporaryFile | ForEach-Object { $_.DirectoryName + "\tempDir" + (Get-Date -Format "yyyyMMddHHmmss") }
New-Item -Path $tempDir -ItemType Directory -Force

Copy-Item "target/release/mouse_speed_profiler.exe" -Destination $tempDir
Copy-Item "README.md" -Destination $tempDir
Copy-Item "README.ja.md" -Destination $tempDir

$zipPath = "$tempDir.zip"
Compress-Archive -Path "$tempDir\*" -DestinationPath $zipPath
Copy-Item $zipPath -Destination "mouse_speed_profiler.zip"
Remove-Item -Path $tempDir -Recurse -Force
