<#
.SYNOPSIS
  Generate the speech WAV fixture for the end-to-end transcription test
  (src/whisper.rs::e2e_transcribes_spoken_digits_from_fixture).

.DESCRIPTION
  Uses the Windows SAPI voice to synthesize a known phrase into a 16 kHz mono
  16-bit PCM WAV — exactly the format whisper-rs consumes — so the E2E test can
  prove the audio->text pipeline without a microphone. Run from the repo root:

      powershell -ExecutionPolicy Bypass -File scripts\gen-test-fixture-windows.ps1

  The committed fixture lets the test run on any Windows machine; regenerate only
  if the expected phrase changes.
#>
$ErrorActionPreference = "Stop"

$phrase = "Testing one two three four. The quick brown fox jumps over the lazy dog."
$dir = Join-Path $PSScriptRoot "..\tests\fixtures"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$path = Join-Path $dir "testing-one-two-three.wav"
Remove-Item $path -ErrorAction SilentlyContinue

$voice  = New-Object -ComObject SAPI.SpVoice
$stream = New-Object -ComObject SAPI.SpFileStream
$format = New-Object -ComObject SAPI.SpAudioFormat
$format.Type = 18   # SAFT16kHz16BitMono — whisper's native input format
$stream.Format = $format
$stream.Open($path, 3, $false)   # SSFMCreateForWrite
$voice.AudioOutputStream = $stream
[void]$voice.Speak($phrase)
$stream.Close()

$b = [System.IO.File]::ReadAllBytes($path)
$channels = [BitConverter]::ToUInt16($b,22)
$rate     = [BitConverter]::ToUInt32($b,24)
$bits     = [BitConverter]::ToUInt16($b,34)
Write-Host "Wrote $path"
Write-Host "  phrase:   $phrase"
Write-Host "  format:   ${channels}ch ${rate}Hz ${bits}-bit  ($($b.Length) bytes)"
if ($channels -ne 1 -or $rate -ne 16000 -or $bits -ne 16) {
  throw "Unexpected WAV format — whisper needs 16 kHz mono 16-bit"
}
