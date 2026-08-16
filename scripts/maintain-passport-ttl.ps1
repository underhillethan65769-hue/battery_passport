param(
  [Parameter(Mandatory = $true)][string]$Serial,
  [Parameter(Mandatory = $true)][int]$AuditCount,
  [string]$IdentityName = "battery_passport_mainnet_operator",
  [string]$RpcUrl = "https://soroban-rpc.mainnet.stellar.gateway.fm"
)

$ErrorActionPreference = "Stop"
$ContractId = "CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5"
$NetworkPassphrase = "Public Global Stellar Network ; September 2015"
$BatchSize = 50

if ([string]::IsNullOrWhiteSpace($Serial)) {
  throw "Serial is required."
}

if ($AuditCount -lt 0) {
  throw "AuditCount cannot be negative."
}

Write-Host "Refreshing Battery Passport TTL for $Serial on Stellar Mainnet" -ForegroundColor Cyan
Write-Host "This submits transactions and spends XLM from identity: $IdentityName" -ForegroundColor Yellow
Write-Host "Never place a secret key in this script or repository." -ForegroundColor Yellow

stellar contract invoke `
  --id $ContractId `
  --source-account $IdentityName `
  --rpc-url $RpcUrl `
  --network-passphrase $NetworkPassphrase `
  -- `
  refresh_passport_ttl `
  --serial $Serial `
  --audit_from 0 `
  --audit_limit 0

if ($LASTEXITCODE -ne 0) {
  throw "Passport TTL refresh failed."
}

for ($Start = 0; $Start -lt $AuditCount; $Start += $BatchSize) {
  stellar contract invoke `
    --id $ContractId `
    --source-account $IdentityName `
    --rpc-url $RpcUrl `
    --network-passphrase $NetworkPassphrase `
    -- `
    refresh_passport_ttl `
    --serial $Serial `
    --audit_from $Start `
    --audit_limit $BatchSize

  if ($LASTEXITCODE -ne 0) {
    throw "Audit TTL refresh failed at audit offset $Start."
  }
}

Write-Host "TTL refresh completed for $Serial." -ForegroundColor Green