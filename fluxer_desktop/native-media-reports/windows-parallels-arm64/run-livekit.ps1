param(
	[string]$Scenario = "vp8-360p15-strict-single",
	[string]$Codec = "vp8",
	[string]$SecondaryCodec = "",
	[int]$Width = 640,
	[int]$Height = 360,
	[double]$Fps = 15,
	[int]$DurationMs = 10000,
	[int]$TimeoutMs = 60000,
	[int]$SecondPublisher = 0,
	[int]$ScreenSimulcast = 0,
	[int]$ScreenAudio = 1,
	[int]$Microphone = 1,
	[int]$SubscriptionCycle = 1,
	[int]$DataPacket = 1,
	[int]$ValidateServerPublishing = 1,
	[int]$RequireStableResolution = 1,
	[int]$MaxPacketLoss = 0,
	[int]$MaxAvDriftMs = 150,
	[int]$MaxFrameGapMs = 250,
	[int]$MaxAudioFrameGapMs = 250,
	[double]$MinReceivedFpsRatio = 0.95,
	[int]$MaxBitrateBps = 4000000,
	[string]$LiveKitUrl = "ws://100.79.83.54:17880",
	[string]$LiveKitApiUrl = "http://100.79.83.54:17880",
	[string]$ApiKey = "devkey",
	[string]$ApiSecret = "secret",
	[string]$NodeDir = "C:\nodejs\node-v24.16.0-win-arm64",
	[string]$WorkDir = "C:\f\fluxer_desktop\native\webrtc-sender",
	[string]$ReportDir = "C:\Mac\Home\Development\fluxer\fluxer_desktop\native-media-reports\windows-parallels-arm64",
	[int]$VerboseHarness = 0
)

$ErrorActionPreference = "Stop"
$nodeExe = Join-Path $NodeDir "node.exe"
if (-not (Test-Path $nodeExe)) {
	throw "Node executable not found: $nodeExe"
}
if (-not (Test-Path $WorkDir)) {
	throw "WorkDir not found: $WorkDir"
}

New-Item -ItemType Directory -Force -Path $ReportDir | Out-Null
$reportPath = Join-Path $ReportDir "$Scenario.json"
$logPath = Join-Path $ReportDir "$Scenario.log"
Remove-Item -Force $reportPath, $logPath -ErrorAction SilentlyContinue

$env:Path = "$NodeDir;$env:Path"
$env:LIVEKIT_URL = $LiveKitUrl
$env:LIVEKIT_API_URL = $LiveKitApiUrl
$env:LIVEKIT_API_KEY = $ApiKey
$env:LIVEKIT_API_SECRET = $ApiSecret
$env:FLUXER_WEBRTC_SENDER_LIVEKIT_REQUIRED = "1"
$env:LIVEKIT_HARNESS_STRICT = "1"
$env:LIVEKIT_HARNESS_DURATION_MS = [string]$DurationMs
$env:LIVEKIT_HARNESS_TIMEOUT_MS = [string]$TimeoutMs
$env:LIVEKIT_CONNECT_TIMEOUT_MS = "30000"
$env:LIVEKIT_SCREEN_WIDTH = [string]$Width
$env:LIVEKIT_SCREEN_HEIGHT = [string]$Height
$env:LIVEKIT_SCREEN_FPS = [string]$Fps
$env:LIVEKIT_MIN_VIDEO_FPS = [string]([math]::Min(15, $Fps))
$env:LIVEKIT_MIN_RECEIVED_FPS_RATIO = [string]$MinReceivedFpsRatio
$env:LIVEKIT_SCREEN_CODEC = $Codec
$env:LIVEKIT_SCREEN_SIMULCAST = [string]$ScreenSimulcast
$env:LIVEKIT_ENABLE_SECOND_PUBLISHER = [string]$SecondPublisher
$env:LIVEKIT_ENABLE_MICROPHONE = [string]$Microphone
$env:LIVEKIT_ENABLE_SCREEN_AUDIO = [string]$ScreenAudio
$env:LIVEKIT_ENABLE_SUBSCRIPTION_CYCLE = [string]$SubscriptionCycle
$env:LIVEKIT_ENABLE_DATA_PACKET = [string]$DataPacket
$env:LIVEKIT_VALIDATE_SERVER_PUBLISHING = [string]$ValidateServerPublishing
$env:LIVEKIT_MAX_PACKET_LOSS = [string]$MaxPacketLoss
$env:LIVEKIT_REQUIRE_STABLE_RESOLUTION = [string]$RequireStableResolution
$env:LIVEKIT_MAX_AV_DRIFT_MS = [string]$MaxAvDriftMs
$env:LIVEKIT_MAX_FRAME_GAP_MS = [string]$MaxFrameGapMs
$env:LIVEKIT_MAX_AUDIO_FRAME_GAP_MS = [string]$MaxAudioFrameGapMs
$env:LIVEKIT_SCREEN_MAX_BITRATE_BPS = [string]$MaxBitrateBps
$env:LIVEKIT_HARNESS_REPORT_PATH = $reportPath
$env:LIVEKIT_VERBOSE = [string]$VerboseHarness

if ($SecondPublisher -ne 0 -and $SecondaryCodec.Trim().Length -gt 0) {
	$env:LIVEKIT_SECOND_PUBLISHER_SCREEN_CODEC = $SecondaryCodec
}

Set-Location $WorkDir
"STARTED=$(Get-Date -Format o)" | Out-File -FilePath $logPath -Encoding utf8
"& $nodeExe scripts/livekit-harness.mjs" | Add-Content -Path $logPath
$previousErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = "Continue"
& $nodeExe "scripts/livekit-harness.mjs" *>> $logPath
$code = $LASTEXITCODE
$ErrorActionPreference = $previousErrorActionPreference
"EXIT_CODE=$code" | Add-Content -Path $logPath
"FINISHED=$(Get-Date -Format o)" | Add-Content -Path $logPath

Get-Content -Path $logPath
exit $code
