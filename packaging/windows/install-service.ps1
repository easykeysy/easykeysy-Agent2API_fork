<#
.SYNOPSIS
    安装或升级 Agent2API Windows 服务。
.DESCRIPTION
    在服务端发行包目录以管理员身份运行。已有安装默认保留程序目录、数据目录、
    端口、服务账号和环境配置；仅显式传入的参数覆盖原值。
.PARAMETER InstallDir
    新安装默认 C:\Agent2API；已有安装默认沿用登记的目录。
.PARAMETER DataDir
    新安装默认 C:\ProgramData\Agent2API；已有安装保留数据位置。
.PARAMETER Port
    新安装默认 3065；已有安装保留原端口。
.PARAMETER Uninstall
    卸载服务，默认保留程序与数据。
.PARAMETER Purge
    配合 -Uninstall 删除程序和数据。
#>
[CmdletBinding()]
param(
    [string]$InstallDir = "C:\Agent2API",
    [string]$DataDir = "C:\ProgramData\Agent2API",
    [ValidateRange(1, 65535)][int]$Port = 3065,
    [switch]$Uninstall,
    [switch]$Purge
)

$ErrorActionPreference = "Stop"
$ServiceName = "Agent2API"
$ExeName = "agent2api-server.exe"
$regPath = "HKLM:\SYSTEM\CurrentControlSet\Services\$ServiceName"
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Administrator rights are required. Open PowerShell as an administrator, then run the install script."
}

function Invoke-ServiceCommand {
    param([string[]]$Arguments)
    & "$env:SystemRoot\System32\sc.exe" @Arguments | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "Service configuration operation failed (exit code$LASTEXITCODE)" }
}

function Wait-ServiceState {
    param([string]$State, [int]$Seconds = 60)
    $service = Get-Service -Name $ServiceName
    $service.WaitForStatus([System.ServiceProcess.ServiceControllerStatus]$State, [TimeSpan]::FromSeconds($Seconds))
    $service.Refresh()
    if ($service.Status.ToString() -ne $State) { throw "Wait for the service to enter$State The state timed out" }
}

function Assert-SafeDirectory {
    param([string]$Path)
    $full = [IO.Path]::GetFullPath($Path)
    if (-not [IO.Path]::IsPathRooted($Path) -or $full.StartsWith('\\') -or
        $full.TrimEnd('\') -eq [IO.Path]::GetPathRoot($full).TrimEnd('\') -or
        $full.TrimEnd('\') -eq $env:SystemRoot.TrimEnd('\')) {
        throw "Unsafe install path:$Path"
    }
    $cursor = $full
    while ($cursor) {
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                throw "The install path must not contain directory links:$cursor"
            }
        }
        $parent = [IO.Directory]::GetParent($cursor)
        if ($null -eq $parent) { break }
        $cursor = $parent.FullName
    }
    return $full.TrimEnd('\')
}

function Set-ProgramPermissions {
    param([string]$Path, [switch]$Private)
    $acl = New-Object Security.AccessControl.DirectorySecurity
    $owner = New-Object Security.Principal.SecurityIdentifier('S-1-5-32-544')
    $acl.SetOwner($owner)
    $acl.SetAccessRuleProtection($true, $false)
    foreach ($sid in @('S-1-5-18', 'S-1-5-32-544')) {
        $identity = New-Object Security.Principal.SecurityIdentifier($sid)
        $rule = New-Object Security.AccessControl.FileSystemAccessRule($identity, 'FullControl', 'ContainerInherit,ObjectInherit', 'None', 'Allow')
        $acl.AddAccessRule($rule)
    }
    if (-not $Private) {
        $users = New-Object Security.Principal.SecurityIdentifier('S-1-5-32-545')
        $rule = New-Object Security.AccessControl.FileSystemAccessRule($users, 'ReadAndExecute', 'ContainerInherit,ObjectInherit', 'None', 'Allow')
        $acl.AddAccessRule($rule)
    }
    Set-Acl -LiteralPath $Path -AclObject $acl
}

$existing = Get-CimInstance Win32_Service -Filter "Name='$ServiceName'" -ErrorAction Stop
$hadService = $null -ne $existing
$environment = @{}
if ($hadService) {
    $stored = Get-ItemProperty -Path $regPath -Name Environment -ErrorAction SilentlyContinue
    foreach ($line in @($stored.Environment)) {
        if ($line -match '^([^=]+)=(.*)$') { $environment[$Matches[1]] = $Matches[2] }
    }
    $command = $existing.PathName.Trim()
    if ($command -match '^"([^"\r\n]+)"\s+--service\s*$') { $installedExe = $Matches[1] }
    elseif ($command -match '^(.+?\.exe)\s+--service\s*$') { $installedExe = $Matches[1] }
    else { throw "The existing service start parameters have been customized; no files were modified. Please upgrade using the original deployment method." }
    if ([IO.Path]::GetFileName($installedExe) -ine $ExeName) {
        throw "The existing service is not an Agent2API server; no files were modified."
    }
    if (-not $PSBoundParameters.ContainsKey('InstallDir')) { $InstallDir = Split-Path -Parent $installedExe }
    if (-not $PSBoundParameters.ContainsKey('DataDir')) {
        if ($environment.ContainsKey('AGENT2API_PROXY_HOME')) { $DataDir = $environment['AGENT2API_PROXY_HOME'] }
        elseif ($environment.ContainsKey('WORKBUDDY_PROXY_HOME')) { $DataDir = $environment['WORKBUDDY_PROXY_HOME'] }
        else { throw 'The old install\'s data directory cannot be confirmed. Specify the original DataDir explicitly to avoid falling back to an empty one.' }
    }
    if (-not $PSBoundParameters.ContainsKey('Port')) {
        if ($environment.ContainsKey('AGENT2API_PROXY_PORT')) { $Port = [int]$environment['AGENT2API_PROXY_PORT'] }
        elseif ($environment.ContainsKey('WORKBUDDY_PROXY_PORT')) { $Port = [int]$environment['WORKBUDDY_PROXY_PORT'] }
    }
}
$InstallDir = Assert-SafeDirectory $InstallDir
$DataDir = Assert-SafeDirectory $DataDir
if ($Port -lt 1 -or $Port -gt 65535) { throw "The service port configuration is invalid; the installation was not modified." }
if (-not $Uninstall -and $hadService -and $PSBoundParameters.ContainsKey('InstallDir') -and
    [IO.Path]::GetFullPath((Split-Path -Parent $installedExe)).TrimEnd('\') -ine $InstallDir) {
    throw 'The upgrade does not migrate the install directory; finish the upgrade first, then migrate per the deployment method.'
}
if (-not $Uninstall -and $hadService -and $PSBoundParameters.ContainsKey('DataDir')) {
    $previousData = $environment['AGENT2API_PROXY_HOME']
    if (-not $previousData) { $previousData = $environment['WORKBUDDY_PROXY_HOME'] }
    if ($previousData -and [IO.Path]::GetFullPath($previousData).TrimEnd('\') -ine $DataDir) {
        throw 'Upgrades do not migrate account data — keep the original DataDir; data migration is a separate step.'
    }
}
$targetExe = Join-Path $InstallDir $ExeName
$targetUi = Join-Path $InstallDir 'ui'
$updateDir = Join-Path $InstallDir '.service-update'
foreach ($path in @($targetExe, $targetUi, $updateDir, (Join-Path $updateDir 'update.lock'))) {
    if (Test-Path -LiteralPath $path) { $null = Assert-SafeDirectory $path }
}

if ($Uninstall) {
    if ($hadService) {
        Stop-Service -Name $ServiceName -ErrorAction Stop
        Wait-ServiceState 'Stopped'
        Invoke-ServiceCommand @('delete', $ServiceName)
    }
    if ($Purge) {
        if (Test-Path -LiteralPath $InstallDir) { Remove-Item -LiteralPath $InstallDir -Recurse -Force }
        if (Test-Path -LiteralPath $DataDir) { Remove-Item -LiteralPath $DataDir -Recurse -Force }
        Write-Host "✅ Uninstalled and cleaned up"
    } else {
        Write-Host "✅ The service has been uninstalled; the program and data are kept:$InstallDir / $DataDir"
    }
    exit 0
}

$sourceExe = Join-Path $PSScriptRoot $ExeName
$sourceUi = Join-Path $PSScriptRoot 'ui'
if (-not (Test-Path -LiteralPath $sourceExe -PathType Leaf)) { throw "The release package is missing$ExeName" }
if ([IO.Path]::GetFullPath($PSScriptRoot).TrimEnd('\') -ieq $InstallDir) {
    throw "Unpack the new package into a fresh directory before running the script; do not run an overwrite upgrade from the existing program directory."
}
$hasUi = Test-Path -LiteralPath (Join-Path $sourceUi 'index.html') -PathType Leaf
if (-not $hasUi) { Write-Warning "The release package has no complete ui directory; only the program is updated, the original UI is not deleted." }
if ($DataDir -ieq $targetUi -or $DataDir.StartsWith($targetUi + '\', [StringComparison]::OrdinalIgnoreCase) -or
    $DataDir -ieq $updateDir -or $DataDir.StartsWith($updateDir + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw "The data directory must not be inside the UI or upgrade working directory being replaced."
}

$isSystem = -not $hadService -or $existing.StartName -in @('LocalSystem', 'NT AUTHORITY\SYSTEM')
$backupExe = $null
$backupUi = $null
$replacedExe = $false
$replacedUi = $false
$wasRunning = $hadService -and $existing.State -eq 'Running'
$originalEnvironment = @($environment.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" })
$startAttempted = $false
$stopRequested = $false
$updateRegistered = $false
$updateLock = $null
try {
    if (Test-Path -LiteralPath $updateDir) {
        $null = Assert-SafeDirectory $updateDir
        $updateLock = [IO.File]::Open((Join-Path $updateDir 'update.lock'), 'OpenOrCreate', 'ReadWrite', 'None')
    }
    if ($hadService) {
        Write-Host '→ Stop the existing service'
        $stopRequested = $true
        Stop-Service -Name $ServiceName -ErrorAction Stop
        Wait-ServiceState 'Stopped'
        if ($existing.ProcessId -gt 0) {
            $process = Get-Process -Id $existing.ProcessId -ErrorAction SilentlyContinue
            if ($null -ne $process -and -not $process.WaitForExit(30000)) { throw 'The old service process has not exited yet; the program was not overwritten.' }
        }
    }
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    if ($isSystem) { Set-ProgramPermissions $InstallDir }
    $suffix = [Guid]::NewGuid().ToString('N')
    $stagedExe = Join-Path $InstallDir ".install-$suffix.exe"
    $stagedUi = Join-Path $InstallDir ".install-ui-$suffix"
    Copy-Item -LiteralPath $sourceExe -Destination $stagedExe
    if ($hasUi) { Copy-Item -LiteralPath $sourceUi -Destination $stagedUi -Recurse }
    if (Test-Path -LiteralPath $targetExe) {
        $backupExe = Join-Path $InstallDir "agent2api-server-$suffix.bak"
        Move-Item -LiteralPath $targetExe -Destination $backupExe
    }
    Move-Item -LiteralPath $stagedExe -Destination $targetExe
    $replacedExe = $true
    if ($hasUi) {
        if (Test-Path -LiteralPath $targetUi) {
            $backupUi = Join-Path $InstallDir "ui-$suffix.bak"
            Move-Item -LiteralPath $targetUi -Destination $backupUi
        }
        Move-Item -LiteralPath $stagedUi -Destination $targetUi
        $replacedUi = $true
    }

    New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $DataDir 'logs') | Out-Null
    $binPath = "`"$targetExe`" --service"
    if ($hadService) {
        Invoke-ServiceCommand @('config', $ServiceName, 'binPath=', $binPath)
    } else {
        Invoke-ServiceCommand @('create', $ServiceName, 'binPath=', $binPath, 'start=', 'auto', 'DisplayName=', 'Agent2API Gateway')
        Invoke-ServiceCommand @('description', $ServiceName, 'Agent2API The headless gateway service')
        Invoke-ServiceCommand @('failure', $ServiceName, 'reset=', '86400', 'actions=', 'restart/5000/restart/10000/restart/30000')
    }

    if (-not $hadService -or $PSBoundParameters.ContainsKey('DataDir') -or -not $environment.ContainsKey('AGENT2API_PROXY_HOME')) {
        $environment['AGENT2API_PROXY_HOME'] = $DataDir
    }
    if (-not $hadService -or $PSBoundParameters.ContainsKey('InstallDir') -or -not $environment.ContainsKey('AGENT2API_UI_DIR')) {
        $environment['AGENT2API_UI_DIR'] = $targetUi
    }
    if (-not $hadService -or $PSBoundParameters.ContainsKey('Port') -or -not $environment.ContainsKey('AGENT2API_PROXY_PORT')) {
        $environment['AGENT2API_PROXY_PORT'] = [string]$Port
    }
    if (-not $environment.ContainsKey('AGENT2API_HOST')) { $environment['AGENT2API_HOST'] = '0.0.0.0' }
    if (-not $environment.ContainsKey('AGENT2API_LOG_FILE') -or $PSBoundParameters.ContainsKey('DataDir')) {
        $environment['AGENT2API_LOG_FILE'] = Join-Path $DataDir 'logs\server.log'
    }
    $environment['AGENT2API_SELF_UPDATE'] = '0'
    $values = @($environment.GetEnumerator() | Sort-Object Key | ForEach-Object { "$($_.Key)=$($_.Value)" })
    New-ItemProperty -Path $regPath -Name Environment -PropertyType MultiString -Value $values -Force | Out-Null

    if ($isSystem -and $hasUi -and $environment['AGENT2API_UI_DIR'] -ieq $targetUi) {
        New-Item -ItemType Directory -Force -Path $updateDir | Out-Null
        Set-ProgramPermissions $updateDir -Private
        if ($null -eq $updateLock) {
            $updateLock = [IO.File]::Open((Join-Path $updateDir 'update.lock'), 'OpenOrCreate', 'ReadWrite', 'None')
        }
        $parameters = "$regPath\Parameters"
        if (-not (Test-Path -LiteralPath $parameters)) {
            New-Item -Path $parameters | Out-Null
        }
        New-ItemProperty -Path $parameters -Name UpdateInstallDir -Value $InstallDir -PropertyType String -Force | Out-Null
        New-ItemProperty -Path $parameters -Name UpdateWorkDir -Value $updateDir -PropertyType String -Force | Out-Null
        $updateRegistered = $true
        Write-Host '→ Panel update component registered; the service account is unchanged'
    } else {
        Write-Warning 'A custom service account or UI location is currently in use; keep the original settings, and do not enable one-click updates.'
    }

    $ruleName = "Agent2API Gateway (TCP $Port)"
    if (-not (Get-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue)) {
        New-NetFirewallRule -DisplayName $ruleName -Direction Inbound -Protocol TCP -LocalPort $Port -Action Allow -Profile Domain,Private | Out-Null
    }
    $startAttempted = $true
    Start-Service -Name $ServiceName
    Wait-ServiceState 'Running' 90
} catch {
    # 新版已启动后可能完成数据库迁移，不自动还原数据库或盲目降级程序。
    $failure = $_
    $current = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    try {
        if ($stopRequested -and $null -ne $current -and -not $startAttempted) {
            if ($current.Status -ne 'Stopped' -and $current.Status -ne 'Running') {
                Wait-ServiceState 'Stopped'
                $current.Refresh()
            }
            if ($current.Status -eq 'Stopped') {
                if ($backupExe -and (Test-Path -LiteralPath $backupExe)) {
                    if ($replacedExe -and (Test-Path -LiteralPath $targetExe)) { Remove-Item -LiteralPath $targetExe -Force }
                    Move-Item -LiteralPath $backupExe -Destination $targetExe -Force
                }
                if ($backupUi -and (Test-Path -LiteralPath $backupUi)) {
                    if ($replacedUi -and (Test-Path -LiteralPath $targetUi)) { Remove-Item -LiteralPath $targetUi -Recurse -Force }
                    Move-Item -LiteralPath $backupUi -Destination $targetUi
                }
                if ($hadService) {
                    Invoke-ServiceCommand @('config', $ServiceName, 'binPath=', $existing.PathName)
                    New-ItemProperty -Path $regPath -Name Environment -PropertyType MultiString -Value $originalEnvironment -Force | Out-Null
                }
                if ($wasRunning) {
                    Start-Service -Name $ServiceName
                    Wait-ServiceState 'Running' 90
                }
            }
        }
    } catch {
        Write-Warning "Restoring the old service did not complete:$($_.Exception.Message)"
    }
    Write-Warning "The upgrade did not complete fully; old files backed up:$backupExe / $backupUi. Account data was not deleted."
    throw $failure
} finally {
    if ($null -ne $updateLock) { $updateLock.Dispose() }
}
Write-Host '✅ The service has started' -ForegroundColor Green
Write-Host "   Panel: http://<local IP>:$Port/"
Write-Host "   Program:$InstallDir"
Write-Host "   Data:$DataDir"
Write-Host "   Log:$($environment['AGENT2API_LOG_FILE'])"
if ($updateRegistered) {
    Write-Host '   Update component registered; whether one-click update works depends on "Settings→ update", per the permission check result.'
    Write-Host '   AGENT2API_WINDOWS_UPDATE=0 An entry that can be turned off.'
} else {
    Write-Host '   This deployment keeps manual upgrades; please keep using the original deployment method.'
}
