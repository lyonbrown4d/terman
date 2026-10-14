param(
    [Parameter(Mandatory)][string]$PreviousMsi,
    [Parameter(Mandatory)][string]$Msi,
    [switch]$AllowHostChanges
)
$ErrorActionPreference = 'Stop'
if (-not $AllowHostChanges) { throw 'Pass -AllowHostChanges on an isolated elevated test host' }
$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Elevation required' }
if (Test-Path 'HKLM:/Software/Terman/Installer') { throw 'Existing Terman installation: preserving prior state by refusing this test' }
foreach ($registryPath in @('HKLM:/Software/Microsoft/Windows/CurrentVersion/Uninstall/*', 'HKLM:/Software/WOW6432Node/Microsoft/Windows/CurrentVersion/Uninstall/*')) {
    if (Get-ItemProperty $registryPath -ErrorAction SilentlyContinue | Where-Object DisplayName -eq 'Terman') {
        throw 'Existing Terman product: refusing host changes'
    }
}
$PreviousMsi = (Resolve-Path -LiteralPath $PreviousMsi).Path
$Msi = (Resolve-Path -LiteralPath $Msi).Path
$id = [guid]::NewGuid().ToString('N').Substring(0, 10)
$userName = "terman_$id"
$screenName = "terman-screen-$id"
$tmuxName = "terman-tmux-$id"
$work = Join-Path ([IO.Path]::GetTempPath()) "terman-installer-test-$id"
$target = Join-Path $work 'installed'
$password = 'T!9a' + [guid]::NewGuid().ToString('N')
$securePassword = ConvertTo-SecureString $password -AsPlainText -Force
$credential = [pscredential]::new("$env:COMPUTERNAME\$userName", $securePassword)
$installer = New-Object -ComObject WindowsInstaller.Installer
[void]$installer.GetType().InvokeMember('UILevel', 'SetProperty', $null, $installer, @(2))
function Get-MsiProperty([string]$Package, [string]$Name) {
    $database = $installer.OpenDatabase($Package, 0)
    $view = $database.OpenView('SELECT `Value` FROM `Property` WHERE `Property`=' + "'$Name'")
    [void]$view.Execute()
    $record = $view.Fetch()
    $value = $record.StringData(1)
    [void]$view.Close()
    $value
}
if ([version](Get-MsiProperty $PreviousMsi 'ProductVersion') -ge [version](Get-MsiProperty $Msi 'ProductVersion') -or
    (Get-MsiProperty $PreviousMsi 'UpgradeCode') -ne (Get-MsiProperty $Msi 'UpgradeCode')) {
    throw 'A real major-upgrade test requires increasing versions with the same UpgradeCode'
}
$installed = $null
$accountCreated = $false
$userSid = $null
. (Join-Path $PSScriptRoot 'windows-path.ps1')
$originalMachinePath = Get-InstallerMachinePath
function Install-Package([string]$Package, [string]$Features) {
    $options = "ADDLOCAL=$Features INSTALLFOLDER=`"$target`" SCREEN_SERVICE_NAME=$screenName TMUX_SERVICE_NAME=$tmuxName"
    if ($Features -like '*Service*') {
        $options += " SERVICEDOMAIN=$env:COMPUTERNAME SERVICEUSERNAME=$userName SERVICEPASSWORD=$password"
    }
    # COM keeps the password out of child-process arguments. MSI hides it in logs.
    try { [void]$installer.InstallProduct($Package, $options) }
    catch { throw 'MSI install failed; inspect Windows Installer event logs (credential values suppressed)' }
    $script:installed = $Package
}
function Run-AsTestUser([string]$Body) {
    $scriptPath = Join-Path $work ('client-' + [guid]::NewGuid().ToString('N') + '.ps1')
    $resultPath = "$scriptPath.result"
    $prefix = "`$ErrorActionPreference='Stop'; `$env:TERMAN_SERVICE_INSTANCE='$id'`n"
    $suffix = "`n'passed' | Set-Content -LiteralPath '$resultPath'"
    Set-Content -LiteralPath $scriptPath -Value ($prefix + $Body + $suffix)
    $process = Start-Process powershell.exe -Credential $credential -LoadUserProfile -WindowStyle Hidden `
        -ArgumentList @('-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', "`"$scriptPath`"") -Wait -PassThru
    if ($process.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $resultPath)) { throw 'Client under service account failed' }
}
function Assert-Services {
    foreach ($name in @($screenName, $tmuxName)) {
        $service = Get-CimInstance Win32_Service -Filter "Name='$name'"
        if (-not $service -or $service.State -ne 'Running' -or $service.ProcessId -eq 0 -or
            $service.StartName -ne "$env:COMPUTERNAME\$userName") { throw "Incorrect broker service state/identity: $name" }
        $process = Get-CimInstance Win32_Process -Filter "ProcessId=$($service.ProcessId)"
        $owner = Invoke-CimMethod -InputObject $process -MethodName GetOwnerSid
        if ($owner.Sid -ne $userSid) { throw 'Broker process is not owned by the test account' }
    }
}
try {
    New-Item -ItemType Directory -Path $work | Out-Null
    New-LocalUser -Name $userName -Password $securePassword -Description 'Disposable Terman installer test' | Out-Null
    $accountCreated = $true
    $userSid = (Get-LocalUser -Name $userName).SID.Value
    & icacls.exe $work /grant "${userName}:(OI)(CI)M" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Could not grant test workspace access' }
    Run-AsTestUser @"
`$data=Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'terman-installer-retention-$id'
New-Item -ItemType Directory -Path `$data | Out-Null
'retain-user-data' | Set-Content (Join-Path `$data 'sentinel.txt')
`$data | Set-Content '$work/data-path.txt'
"@
    $dataPath = (Get-Content (Join-Path $work 'data-path.txt')).Trim()
    $sentinel = Join-Path $dataPath 'sentinel.txt'
    $originalHash = (Get-FileHash -LiteralPath $sentinel).Hash
    Install-Package $PreviousMsi 'Htop'
    Assert-InstallerPath -Baseline $originalMachinePath -InstallDirectory $target -Present $true
    Run-AsTestUser (Get-InstallerCommandCheck -InstallDirectory $target)
    if (-not (Test-Path (Join-Path $target 'terman-htop.exe')) -or
        (Test-Path (Join-Path $target 'terman-screen.exe')) -or
        (Get-Service -Name $screenName, $tmuxName -ErrorAction SilentlyContinue)) { throw 'Independent Htop feature failed' }
    Install-Package $PreviousMsi 'Screen,Tmux,Htop,ScreenService,TmuxService'
    Assert-InstallerPath -Baseline $originalMachinePath -InstallDirectory $target -Present $true
    Assert-Services
    Run-AsTestUser @"
& '$target/terman-screen.exe' -d -S installer-test --command 'ping -t 127.0.0.1'
if (`$LASTEXITCODE -ne 0) { throw 'Screen session create failed' }
& '$target/terman-screen.exe' -S installer-test -X info
if (`$LASTEXITCODE -ne 0) { throw 'Screen broker IPC failed' }
& '$target/terman-screen.exe' -S installer-test -X quit
& '$target/terman-tmux.exe' new-session -d -s installer-test 'ping -t 127.0.0.1'
if (`$LASTEXITCODE -ne 0) { throw 'Tmux session create failed' }
& '$target/terman-tmux.exe' list-sessions
if (`$LASTEXITCODE -ne 0) { throw 'Tmux broker IPC failed' }
& '$target/terman-tmux.exe' kill-session -t installer-test
"@
    Install-Package $Msi 'Screen,Tmux,Htop,ScreenService,TmuxService'
    Assert-InstallerPath -Baseline $originalMachinePath -InstallDirectory $target -Present $true
    Run-AsTestUser (Get-InstallerCommandCheck -InstallDirectory $target)
    Assert-Services
    $oldCode = Get-MsiProperty $PreviousMsi 'ProductCode'
    if ($installer.ProductState($oldCode) -eq 5) { throw 'Major upgrade left the previous product installed' }
    if ((Get-FileHash -LiteralPath $sentinel).Hash -ne $originalHash) { throw 'Upgrade modified user data' }
    [void]$installer.InstallProduct($Msi, 'REMOVE=ALL')
    $installed = $null
    Assert-InstallerPath -Baseline $originalMachinePath -InstallDirectory $target -Present $false
    if (Get-Service -Name $screenName, $tmuxName -ErrorAction SilentlyContinue) { throw 'Uninstall left broker services' }
    if ((Get-FileHash -LiteralPath $sentinel).Hash -ne $originalHash) { throw 'Uninstall modified user data' }
    if (Test-Path (Join-Path $target 'terman-htop.exe')) { throw 'Uninstall left packaged files' }
    Write-Host 'MSI feature selection, native services, upgrade, uninstall and retention passed'
} finally {
    if ($installed) { try { [void]$installer.InstallProduct($installed, 'REMOVE=ALL') } catch { Write-Warning 'Test MSI cleanup failed' } }
    if ($accountCreated) {
        # Remove only the disposable account/profile identified by the generated SID.
        for ($attempt = 0; $attempt -lt 10; $attempt++) {
            $profile = Get-CimInstance Win32_UserProfile | Where-Object SID -eq $userSid
            if (-not $profile -or -not $profile.Loaded) { break }
            Start-Sleep -Milliseconds 500
        }
        if ($profile -and -not $profile.Loaded) { $profile | Remove-CimInstance }
        if ($profile -and $profile.Loaded) { Write-Warning 'Disposable profile is still loaded; retaining it for safe cleanup' }
        Remove-LocalUser -Name $userName
    }
    $resolved = [IO.Path]::GetFullPath($work)
    if ($resolved.StartsWith([IO.Path]::GetFullPath([IO.Path]::GetTempPath()), [StringComparison]::OrdinalIgnoreCase) -and
        [IO.Path]::GetFileName($resolved) -eq "terman-installer-test-$id") {
        Remove-Item -LiteralPath $resolved -Recurse -Force -ErrorAction SilentlyContinue
    }
    $password = $null
}
