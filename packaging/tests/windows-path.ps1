# Read-only assertions for the native MSI environment-table lifecycle.
function Get-InstallerMachinePath {
    $key = [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey('SYSTEM\CurrentControlSet\Control\Session Manager\Environment')
    if (-not $key) { throw 'Machine environment registry key is unavailable' }
    try {
        return [string]$key.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
    } finally { $key.Dispose() }
}
function Assert-InstallerPath {
    param([string]$Baseline, [string]$InstallDirectory, [bool]$Present)
    $directory = $InstallDirectory.TrimEnd('\')
    $items = @((Get-InstallerMachinePath).Split(';') | Where-Object { $_ -ne '' })
    $matches = @($items | Where-Object { $_.TrimEnd('\') -ieq $directory })
    $expected = if ($Present) { 1 } else { 0 }
    if ($matches.Count -ne $expected) { throw 'Install/upgrade/uninstall left an incorrect number of Terman PATH entries' }
    $other = @($items | Where-Object { $_.TrimEnd('\') -ine $directory })
    $original = @($Baseline.Split(';') | Where-Object { $_ -ne '' })
    if (-not [string]::Equals(($other -join ';'), ($original -join ';'), [StringComparison]::Ordinal)) {
        throw 'Installer modified unrelated PATH entries or their order'
    }
}
function Get-InstallerCommandCheck {
    param([string]$InstallDirectory)
    $expected = (Join-Path $InstallDirectory 'terman-htop.exe').Replace("'", "''")
    # Existing parent processes keep their old environment. Simulate a fresh
    # terminal's PATH without writing any environment variables to the registry.
    return @"
`$env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')
`$command = Get-Command terman-htop.exe -CommandType Application -ErrorAction Stop
if (`$command.Source -ine '$expected') { throw 'Bare command did not resolve to the installed tool' }
& terman-htop.exe --version
if (`$LASTEXITCODE -ne 0) { throw 'Bare installed command failed' }
"@
}
