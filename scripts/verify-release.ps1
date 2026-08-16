param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)

$ErrorActionPreference = "Stop"

$ContractId = "CB2SENPKHCYERH3WF3ILL7Q3RGFPDBYOC4WC4WSMNZUOU6KTXOX5PVQ5"
$ExpectedWasmHash = "0e7f3ce1012e76b877ef021c2d8de5aef0303a509aa3138dfab4f8ae60347663"
$RpcUrl = "https://soroban-rpc.mainnet.stellar.gateway.fm"
$Passphrase = "Public Global Stellar Network ; September 2015"
$ReadAccount = "GANXWLW37X7D6FHGHPOQZGYQRF5G5EYAALQX4R6FNS2XT3B4UPTYL63L"

function Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "=== $Message ==="
}

function Assert-Native {
    param([string]$Message)

    if ($LASTEXITCODE -ne 0) {
        throw "$Message Exit code: $LASTEXITCODE"
    }
}

Set-Location $RepoRoot

Step "Check required production files"

$Required = @(
    "Cargo.toml",
    "Cargo.lock",
    "contracts/battery_passport/Cargo.toml",
    "contracts/battery_passport/src/lib.rs",
    "contracts/battery_passport/src/test.rs",
    "frontend/package.json",
    "frontend/package-lock.json",
    "frontend/src/App.tsx",
    "frontend/src/contractConfig.ts",
    "frontend/src/services/contract.ts",
    "deployments/mainnet.json",
    "docs/ARCHITECTURE.md",
    "docs/MAINNET_DEPLOYMENT.md",
    "docs/QUALITY_AND_DEPLOYMENT.md",
    "docs/SECURITY_REVIEW.md",
    "docs/USER_GUIDE.md",
    ".github/workflows/ci.yml",
    "README.md",
    "vercel.json"
)

foreach ($File in $Required) {
    if (-not (Test-Path $File)) {
        throw "Missing required file: $File"
    }

    Write-Host "OK: $File"
}

Step "Check Rust formatting"

& cargo fmt --all -- --check
Assert-Native "Rust formatting failed."

Step "Run contract tests"

& cargo test --workspace
Assert-Native "Contract tests failed."

Step "Build locked production WASM"

if (-not (Get-Command stellar -ErrorAction SilentlyContinue)) {
    throw "Stellar CLI is required."
}

& stellar contract build --locked
Assert-Native "Stellar contract build failed."

$WasmPath = Join-Path $RepoRoot "target\wasm32v1-none\release\battery_passport.wasm"

if (-not (Test-Path $WasmPath)) {
    throw "Built WASM not found: $WasmPath"
}

$ActualWasmHash = (Get-FileHash $WasmPath -Algorithm SHA256).Hash.ToLowerInvariant()

if ($ActualWasmHash -ne $ExpectedWasmHash) {
    throw "Local WASM mismatch. Expected $ExpectedWasmHash but built $ActualWasmHash"
}

Write-Host "Local WASM hash verified: $ActualWasmHash"

Step "Verify Mainnet contract WASM"

$HashOutput = @(
    & stellar contract info hash `
        --contract-id $ContractId `
        --rpc-url $RpcUrl `
        --network-passphrase $Passphrase
)

Assert-Native "Could not read deployed Mainnet contract hash."

$OnChainHash = (
    $HashOutput |
    Where-Object { $_ -match '^[0-9a-fA-F]{64}$' } |
    Select-Object -Last 1
).Trim().ToLowerInvariant()

if ($OnChainHash -ne $ExpectedWasmHash) {
    throw "Mainnet WASM mismatch. Expected $ExpectedWasmHash but got $OnChainHash"
}

Write-Host "Mainnet WASM hash verified: $OnChainHash"

Step "Verify deployed Mainnet interface"

$Interface = (
    & stellar contract info interface `
        --contract-id $ContractId `
        --rpc-url $RpcUrl `
        --network-passphrase $Passphrase
) -join "
"

Assert-Native "Could not read deployed contract interface."

$RequiredFunctions = @(
    "__constructor",
    "create_passport",
    "add_inspection",
    "verify_passport",
    "transfer_owner",
    "flag_recall",
    "request_recycling",
    "approve_recycling",
    "execute_recycling",
    "get_passport",
    "get_stats",
    "get_audit",
    "get_recent_audits",
    "get_recycling_approval",
    "grant_role",
    "revoke_role",
    "transfer_admin",
    "refresh_passport_ttl",
    "refresh_role_ttl"
)

foreach ($Function in $RequiredFunctions) {
    if ($Interface -notmatch "fn\s+$([regex]::Escape($Function))\s*\(") {
        throw "Mainnet interface missing required function: $Function"
    }

    Write-Host "OK interface: $Function"
}

Step "Check frontend"

Push-Location (Join-Path $RepoRoot "frontend")
try {
    & npm ci
    Assert-Native "Frontend npm ci failed."

    & npm run type-check
    Assert-Native "Frontend type-check failed."

    & npm run build
    Assert-Native "Frontend production build failed."
}
finally {
    Pop-Location
}

Step "Check Mainnet runtime configuration"

$Config = Get-Content "frontend/src/contractConfig.ts" -Raw
$Deployment = Get-Content "deployments/mainnet.json" -Raw

foreach ($Expected in @(
    $ContractId,
    $RpcUrl,
    $Passphrase,
    $ReadAccount
)) {
    if (-not $Config.Contains($Expected)) {
        throw "Missing Mainnet frontend value: $Expected"
    }
}

foreach ($Expected in @(
    $ContractId,
    $ExpectedWasmHash,
    $ReadAccount
)) {
    if (-not $Deployment.Contains($Expected)) {
        throw "Missing verified deployment value: $Expected"
    }
}

Step "Reject stale pre-Mainnet artifacts"

foreach ($Removed in @(
    "frontend/.env.example",
    "frontend/.env.local",
    "scripts/deploy-mainnet.ps1",
    "scripts/deploy-testnet.ps1",
    "scripts/testnet-e2e.ps1",
    ".github/workflows/testnet-e2e.yml",
    "docs/TESTNET_REGRESSION.md",
    "TESTNET_CONTRACT_ID.txt",
    "TESTNET_E2E_REPORT.txt",
    "MAINNET_CONTRACT_ID.txt"
)) {
    if (Test-Path $Removed) {
        throw "Stale pre-Mainnet/local deployment file still exists: $Removed"
    }
}

$ScanRoots = @(
    "README.md",
    "CONTRIBUTING.md",
    "docs",
    "frontend",
    "scripts",
    "deployments"
)

$Files = foreach ($Root in $ScanRoots) {
    if (Test-Path $Root -PathType Leaf) {
        Get-Item $Root
    }
    elseif (Test-Path $Root) {
        Get-ChildItem $Root -File -Recurse |
            Where-Object {
                $_.FullName -notmatch '\\node_modules\\' -and
                $_.FullName -notmatch '\\dist\\' -and
                $_.FullName -notmatch '\\.vite\\' -and
                $_.Name -ne "package-lock.json" -and
                $_.Name -ne "verify-release.ps1"
            }
    }
}

$StalePatterns = @(
    "soroban-testnet",
    "Test SDF Network",
    "VITE_STELLAR_NETWORK=TESTNET",
    "Testnet regression",
    "Testnet E2E",
    "structured for a fresh Mainnet deployment",
    "Mainnet is blocked"
)

foreach ($Pattern in $StalePatterns) {
    $Matches = $Files | Select-String -SimpleMatch $Pattern

    if ($Matches) {
        $Matches | ForEach-Object {
            Write-Host "$($_.Path):$($_.LineNumber): $($_.Line)"
        }

        throw "Stale pre-Mainnet content detected: $Pattern"
    }
}

Step "Check tracked secrets/generated artifacts"

if (Test-Path ".git") {
    $Tracked = @(& git ls-files)
    Assert-Native "git ls-files failed."

    $Forbidden = $Tracked | Where-Object {
        $_ -match '(^|/)\.env($|\.)' -or
        $_ -match '\.xdr$' -or
        $_ -match '\.log$' -or
        $_ -match '(^|/)target/' -or
        $_ -match '(^|/)node_modules/' -or
        $_ -match '(^|/)dist/' -or
        $_ -match '(^|/)test_snapshots/'
    }

    if ($Forbidden) {
        Write-Host "Forbidden tracked artifacts:"
        $Forbidden | ForEach-Object { Write-Host $_ }
        throw "Sensitive/generated artifacts are tracked."
    }
}
else {
    Write-Host "No .git directory yet; tracked-file scan skipped."
}

Step "Check secret-like Stellar values"

$SecretScanFiles = Get-ChildItem . -File -Recurse |
    Where-Object {
        $_.FullName -notmatch '\\.git\\' -and
        $_.FullName -notmatch '\\node_modules\\' -and
        $_.FullName -notmatch '\\target\\' -and
        $_.FullName -notmatch '\\dist\\' -and
        $_.Name -ne "package-lock.json"
    }

$SecretMatches = $SecretScanFiles | Select-String -Pattern 'S[A-Z2-7]{55}'

if ($SecretMatches) {
    $SecretMatches | ForEach-Object {
        Write-Host "$($_.Path):$($_.LineNumber): $($_.Line)"
    }

    throw "Stellar secret-like value detected."
}

Write-Host ""
Write-Host "=== Battery Passport Mainnet release verification passed ===" -ForegroundColor Green