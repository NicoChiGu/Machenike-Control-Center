$proc = Start-Process -FilePath "D:\NodeJSProject\Machenike_L16W_Center\MachenikeControlCenter.exe" -PassThru
Start-Sleep -Seconds 3
$p = Get-Process -Id $proc.Id -ErrorAction SilentlyContinue
if ($p) {
    Write-Host "Process running successfully! ID: $($p.Id), Responding: $($p.Responding), Memory: $([Math]::Round($p.WorkingSet64 / 1MB, 2)) MB"
} else {
    Write-Host "Process exited unexpectedly."
}
