param([Parameter(Mandatory=$true)][string]$GameRoot)
$ErrorActionPreference = 'Stop'
$project = Split-Path -Parent $PSScriptRoot
$source = Join-Path $project 'dist/Better_Skillinfo'
$manifest = Get-Content -LiteralPath (Join-Path $project 'dist/manifest_sha256.json') -Raw | ConvertFrom-Json
if (Get-Process -Name mhgurecomp -ErrorAction SilentlyContinue) { throw '게임을 저장하고 종료한 뒤 설치하세요.' }
foreach ($entry in $manifest.PSObject.Properties) {
    $hash = (Get-FileHash -LiteralPath (Join-Path $source $entry.Name) -Algorithm SHA256).Hash
    if ($hash -ne $entry.Value) { throw "배포 파일 해시 불일치: $($entry.Name)" }
}
$destination = Join-Path $GameRoot 'mods/Better_Skillinfo'
foreach ($entry in $manifest.PSObject.Properties) {
    $target = Join-Path $destination $entry.Name
    New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $source $entry.Name) -Destination $target -Force
    if ((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash -ne $entry.Value) { throw "설치 검증 실패: $target" }
}
$order = Join-Path $GameRoot 'mods/load_order.txt'
if (Test-Path -LiteralPath $order) {
    $lines = @(Get-Content -LiteralPath $order | Where-Object { $_ -notin @('Better_Skillinfo','# off: Better_Skillinfo') })
    @($lines) + 'Better_Skillinfo' | Set-Content -LiteralPath $order -Encoding utf8
}
Write-Output "설치 및 해시 검증 완료: $destination"
