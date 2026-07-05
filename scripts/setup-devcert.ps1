# setup-devcert.ps1
# Run once as Administrator.
# - Creates code-signing cert in CurrentUser\My (private key accessible without elevation)
# - Imports public cert into LocalMachine\TrustedPublisher and LocalMachine\Root (needs Admin)
# - Saves thumbprint for build.sh to use

$ErrorActionPreference = "Stop"

$subject   = "CN=VidCut Dev Signing"
$cerPath   = "$env:TEMP\vidcut-dev.cer"
$thumbFile = Join-Path $PSScriptRoot "dev-cert-thumbprint.txt"

# --- Create cert in CurrentUser\My (private key accessible to current user) ---
$existing = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq $subject -and $_.HasPrivateKey }
if ($existing) {
    Write-Host "Dev cert already exists in CurrentUser\My: $($existing.Thumbprint)" -ForegroundColor Yellow
    $cert = $existing
} else {
    Write-Host "Creating self-signed code-signing certificate in CurrentUser\My ..." -ForegroundColor Cyan
    $cert = New-SelfSignedCertificate `
        -Subject $subject `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -Type CodeSigningCert `
        -KeyUsage DigitalSignature `
        -KeySpec Signature `
        -HashAlgorithm SHA256 `
        -NotAfter (Get-Date).AddYears(10)
    Write-Host "Created cert: $($cert.Thumbprint)" -ForegroundColor Green
}

$thumb = $cert.Thumbprint

# --- Export public cert and install to LocalMachine trust stores (requires Admin) ---
Export-Certificate -Cert "Cert:\CurrentUser\My\$thumb" -FilePath $cerPath | Out-Null

Write-Host "Installing to LocalMachine\TrustedPublisher ..." -ForegroundColor Cyan
Import-Certificate -FilePath $cerPath -CertStoreLocation "Cert:\LocalMachine\TrustedPublisher" | Out-Null

Write-Host "Installing to LocalMachine\Root ..." -ForegroundColor Cyan
Import-Certificate -FilePath $cerPath -CertStoreLocation "Cert:\LocalMachine\Root" | Out-Null

# --- Save thumbprint for build.sh ---
Set-Content -Path $thumbFile -Value $thumb -NoNewline
Write-Host ""
Write-Host "Done! Thumbprint: $thumb" -ForegroundColor Green
Write-Host "Saved to: $thumbFile"
Write-Host ""
Write-Host "build.sh will now sign VidCut.exe after each build using Set-AuthenticodeSignature." -ForegroundColor Green
