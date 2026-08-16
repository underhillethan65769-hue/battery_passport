param(
  [Parameter(Mandatory = $true)][string]$Account,
  [string]$IdentityName = "battery_passport_mainnet_operator",
  [string]$RpcUrl = "https://soroban-rpc.mainnet.stellar.gateway.fm"
)

$ErrorActionPreference = "Stop"
$ContractId = "CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5"
$NetworkPassphrase = "Public Global Stellar Network ; September 2015"

if ($Account -notmatch '^G[A-Z2-7]{55}$') {
  throw "Account must be a public Stellar G-address."
}

Write-Host "Refreshing Battery Passport role TTL on Stellar Mainnet" -ForegroundColor Cyan
Write-Host "This submits a transaction and spends XLM from identity: $IdentityName" -ForegroundColor Yellow

stellar contract invoke `
  --id $ContractId `
  --source-account $IdentityName `
  --rpc-url $RpcUrl `
  --network-passphrase $NetworkPassphrase `
  -- `
  refresh_role_ttl `
  --account $Account

if ($LASTEXITCODE -ne 0) {
  throw "Role TTL refresh failed."
}

Write-Host "Role TTL refresh completed." -ForegroundColor Green