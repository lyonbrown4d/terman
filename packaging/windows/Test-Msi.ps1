param([Parameter(Mandatory)][string]$Path)
$ErrorActionPreference = 'Stop'
$installer = New-Object -ComObject WindowsInstaller.Installer
$database = $installer.OpenDatabase((Resolve-Path -LiteralPath $Path).Path, 0)
function Read-Rows([string]$Sql) {
    $view = $database.OpenView($Sql)
    [void]$view.Execute()
    $record = $view.Fetch()
    while ($null -ne $record) {
        $row = @()
        $count = $record.GetType().InvokeMember('FieldCount', 'GetProperty', $null, $record, $null)
        for ($i = 1; $i -le $count; $i++) { $row += $record.StringData($i) }
        ,$row
        $record = $view.Fetch()
    }
    [void]$view.Close()
}
$services = @(Read-Rows 'SELECT `Name`, `StartName`, `Password`, `Arguments` FROM `ServiceInstall`')
if ($services.Count -ne 2) { throw 'Expected exactly two broker services' }
foreach ($row in $services) {
    if ($row[0] -notin @('[SCREEN_SERVICE_NAME]', '[TMUX_SERVICE_NAME]') -or
        $row[1] -ne '[SERVICEDOMAIN]\[SERVICEUSERNAME]' -or
        $row[2] -ne '[SERVICEPASSWORD]' -or $row[3] -ne ('service run --name "' + $row[0] + '"')) {
        throw 'Unexpected service identity or invocation'
    }
}
$controls = @(Read-Rows 'SELECT `Name`, `Event`, `Wait` FROM `ServiceControl`')
if ($controls.Count -ne 2) { throw 'Expected native service lifecycle controls' }
foreach ($row in $controls) {
    # Start on install, stop on install/uninstall, delete on uninstall.
    if ([int]$row[1] -ne 163 -or [int]$row[2] -ne 1) { throw 'Incorrect native service lifecycle' }
}
$features = @(Read-Rows 'SELECT `Feature`, `Level` FROM `Feature`')
foreach ($name in @('Screen', 'Tmux', 'Htop', 'ScreenService', 'TmuxService')) {
    $row = $features | Where-Object { $_[0] -eq $name }
    if (-not $row) { throw "Missing independent feature $name" }
    if ($name -like '*Service' -and [int]$row[1] -ne 2) { throw 'Services must be opt-in' }
}
$properties = @(Read-Rows 'SELECT `Property`, `Value` FROM `Property`')
$hidden = $properties | Where-Object { $_[0] -eq 'MsiHiddenProperties' }
if (-not $hidden -or 'SERVICEPASSWORD' -notin $hidden[1].Split(';')) {
    throw 'Service password is not hidden from MSI logging'
}
$files = @(Read-Rows 'SELECT `FileName` FROM `File`')
if ($files.Count -ne 5) { throw 'Expected three tools and two optional broker executables' }
$permissions = @(Read-Rows 'SELECT * FROM `Wix4SecureObject`')
if ($permissions.Count -ne 2) { throw 'Expected service account permissions for both brokers' }
foreach ($row in $permissions) {
    if ($row[1] -ne 'ServiceInstall' -or $row[2] -ne '[SERVICEDOMAIN]' -or
        $row[3] -ne '[SERVICEUSERNAME]' -or [int]$row[5] -ne 131125) {
        throw 'Service account permissions must allow start/stop/query without configuration changes'
    }
}
Write-Host "Validated MSI tables: $Path"
