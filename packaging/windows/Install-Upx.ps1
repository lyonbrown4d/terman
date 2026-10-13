$ErrorActionPreference = 'Stop'
$release = Invoke-RestMethod 'https://api.github.com/repos/upx/upx/releases/latest'
$asset = $release.assets | Where-Object name -like '*-win64.zip' | Select-Object -First 1
if (-not $asset -or $asset.digest -notlike 'sha256:*') { throw 'Missing UPX asset checksum' }
$zipPath = Join-Path $env:RUNNER_TEMP 'upx-win64.zip'
$toolPath = Join-Path $env:RUNNER_TEMP 'upx-tool'
Invoke-WebRequest $asset.browser_download_url -OutFile $zipPath
$digest = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($digest -ne $asset.digest.Substring(7)) { throw 'UPX checksum mismatch' }
Expand-Archive -LiteralPath $zipPath -DestinationPath $toolPath
$upxExe = Get-ChildItem -LiteralPath $toolPath -Recurse -Filter upx.exe | Select-Object -First 1
if (-not $upxExe) { throw 'UPX executable missing' }
$upxExe.DirectoryName | Out-File $env:GITHUB_PATH -Append -Encoding utf8
