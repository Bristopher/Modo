param(
	[Parameter(Mandatory = $true)][string]$Command,
	[string]$OutFile = 'C:\tools\vm_test_out.txt',
	[int]$TimeoutSec = 120
)

$taskName = 'FluxerVmTest'
if (Test-Path $OutFile) { Remove-Item -Force $OutFile }
Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue

$action = New-ScheduledTaskAction -Execute 'cmd.exe' -Argument ('/c ' + $Command + ' > "' + $OutFile + '" 2>&1')
$principal = New-ScheduledTaskPrincipal -UserId 'hampus' -LogonType Interactive
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit (New-TimeSpan -Minutes 10)
Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal -Settings $settings -Force | Out-Null
Start-ScheduledTask -TaskName $taskName

$deadline = (Get-Date).AddSeconds($TimeoutSec)
do {
	Start-Sleep -Milliseconds 500
	$state = (Get-ScheduledTask -TaskName $taskName).State
} while ($state -ne 'Ready' -and (Get-Date) -lt $deadline)

$info = Get-ScheduledTaskInfo -TaskName $taskName
Write-Output ('TASK-STATE: ' + $state + ' LAST-RESULT: ' + $info.LastTaskResult)
if (Test-Path $OutFile) {
	Write-Output '--- OUTPUT ---'
	Get-Content $OutFile
} else {
	Write-Output 'NO OUTPUT FILE'
}
Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue
