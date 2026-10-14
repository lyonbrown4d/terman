param(
    [Parameter(Mandatory)][string]$ReleaseDirectory,
    [string]$Version,
    [ValidateSet('amd64', 'arm64')][string[]]$Architecture = @('amd64', 'arm64'),
    [string]$WixPath = 'wix',
    [switch]$AcceptWixEula
)
$ErrorActionPreference = 'Stop'
$ReleaseDirectory = (Resolve-Path -LiteralPath $ReleaseDirectory).Path
$repository = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
if ($ReleaseDirectory.Equals($repository, [StringComparison]::OrdinalIgnoreCase) -or
    $ReleaseDirectory.StartsWith($repository + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Release output must be outside the repository'
}
$metadata = Get-Content (Join-Path $ReleaseDirectory 'metadata.json') -Raw | ConvertFrom-Json
$artifactVersion = $metadata.version
if (-not $Version) {
    if ($artifactVersion -notmatch '^(\d+)\.(\d+)\.(\d+)') { throw 'Version must start with major.minor.patch' }
    $Version = "$($Matches[1]).$($Matches[2]).$($Matches[3])"
}
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw 'MSI version must have three numeric fields' }
$parts = $Version.Split('.') | ForEach-Object { [int]$_ }
if ($parts[0] -gt 255 -or $parts[1] -gt 255 -or $parts[2] -gt 65535) { throw 'MSI version exceeds Windows Installer limits' }
$work = Join-Path ([IO.Path]::GetTempPath()) ('terman-msi-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null
try {
    foreach ($arch in $Architecture) {
        $zip = Join-Path $ReleaseDirectory "terman_${artifactVersion}_windows_${arch}.zip"
        $payload = Join-Path $work $arch
        Expand-Archive -LiteralPath $zip -DestinationPath $payload
        foreach ($tool in @('screen', 'tmux', 'htop')) {
            $exe = Join-Path $payload "terman-$tool.exe"
            & upx --test $exe
            if ($LASTEXITCODE -ne 0) { throw "UPX validation failed: $tool/$arch" }
        }
        $wixArch = if ($arch -eq 'amd64') { 'x64' } else { 'arm64' }
        $upgrade = if ($arch -eq 'amd64') { '4A8C17DD-9847-453D-81A4-E73223B75091' } else { '2DC910E8-03C1-411B-ACF0-20DC2F92AEC1' }
        $msi = Join-Path $ReleaseDirectory "terman_${artifactVersion}_windows_${arch}.msi"
        $eulaArguments = @()
        if ($AcceptWixEula) { $eulaArguments = @('-acceptEula', 'wix7') }
        & $WixPath build @eulaArguments -arch $wixArch -ext WixToolset.Util.wixext/7.0.0 `
            -d "Version=$Version" -d "UpgradeCode=$upgrade" -d "Payload=$payload" `
            -intermediateFolder (Join-Path $work "$arch-obj") `
            -o $msi (Join-Path $PSScriptRoot 'terman.wxs') (Join-Path $PSScriptRoot 'ui.wxs') (Join-Path $PSScriptRoot 'path.wxs')
        if ($LASTEXITCODE -ne 0) { throw "WiX failed: $arch" }
        & (Join-Path $PSScriptRoot 'Test-Msi.ps1') -Path $msi
        $hash = (Get-FileHash -LiteralPath $msi -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $([IO.Path]::GetFileName($msi))" | Set-Content -LiteralPath "$msi.sha256" -Encoding ascii
    }
} finally {
    # The only recursively removed path is the generated temp workspace above.
    $resolved = [IO.Path]::GetFullPath($work)
    $tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    if ($resolved.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase) -and
        [IO.Path]::GetFileName($resolved) -like 'terman-msi-*') {
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
}
