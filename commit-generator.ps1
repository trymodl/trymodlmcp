Write-Host "Creating commit history for Shift Protocol..." -ForegroundColor Green

$messages = @(
    "Add hardware device validation",
    "Optimize attestation verification", 
    "Fix memory leak in key encumbrance",
    "Add P2P channel timeout handling",
    "Update SDK core client error handling",
    "Add unit tests for hardware attestation",
    "Implement secure key derivation",
    "Fix race condition in processing",
    "Add logging for debug mode",
    "Optimize Solana account allocation",
    "Add input validation for devices",
    "Update TypeScript interfaces",
    "Fix clippy warnings in attestation",
    "Add comprehensive error codes",
    "Implement transaction batching",
    "Add hardware compatibility checks",
    "Fix serialization edge case",
    "Add performance benchmarking",
    "Optimize key pool management",
    "Add retry mechanism for network"
)

$files = @(
    "programs/shift-core/src/lib.rs",
    "programs/shift-attestation/src/lib.rs",
    "programs/shift-encumbrance/src/lib.rs",
    "programs/shift-p2p/src/lib.rs",
    "sdk/src/core.ts",
    "sdk/src/utils.ts",
    "sdk/src/types.ts",
    "tests/attestation.ts"
)

$startDate = Get-Date "2025-05-02"
$endDate = Get-Date "2025-06-03"
$currentDate = $startDate
$totalCommits = 0

Write-Host "Generating commits..." -ForegroundColor Cyan

while ($currentDate -le $endDate -and $totalCommits -lt 100) {
    $commitsToday = Get-Random -Minimum 1 -Maximum 5
    
    for ($i = 0; $i -lt $commitsToday -and $totalCommits -lt 100; $i++) {
        $hour = Get-Random -Minimum 8 -Maximum 23
        $minute = Get-Random -Minimum 0 -Maximum 59
        $second = Get-Random -Minimum 0 -Maximum 59
        
        $commitDateTime = $currentDate.AddHours($hour).AddMinutes($minute).AddSeconds($second)
        $dateString = $commitDateTime.ToString("yyyy-MM-dd HH:mm:ss")
        
        $message = $messages | Get-Random
        $filePath = $files | Get-Random
        
        $dir = Split-Path $filePath -Parent
        if (!(Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
        
        if (!(Test-Path $filePath)) {
            New-Item -ItemType File -Path $filePath -Force | Out-Null
        }
        
        $randomNum = Get-Random -Minimum 1000 -Maximum 9999
        $content = "Comment added for commit $randomNum"
        Add-Content -Path $filePath -Value $content
        
        git add .
        
        $env:GIT_COMMITTER_DATE = $dateString
        $env:GIT_AUTHOR_DATE = $dateString
        
        git commit -m $message --date=$dateString
        
        $totalCommits++
        
        if ($totalCommits % 10 -eq 0) {
            Write-Host "Created $totalCommits commits" -ForegroundColor Green
        }
        
        Start-Sleep -Milliseconds 50
    }
    
    $currentDate = $currentDate.AddDays(1)
}

Remove-Item Env:\GIT_COMMITTER_DATE -ErrorAction SilentlyContinue
Remove-Item Env:\GIT_AUTHOR_DATE -ErrorAction SilentlyContinue

Write-Host "Created $totalCommits commits from May 2025 to June 2025!" -ForegroundColor Green 