param([Parameter(Mandatory=$true)][string]$GameRoot)
$ErrorActionPreference = 'Stop'
$project = Split-Path -Parent $PSScriptRoot
$source = Join-Path $project 'dist/Better-Skillinfo'
$manifest = Get-Content -LiteralPath (Join-Path $project 'dist/manifest_sha256.json') -Raw | ConvertFrom-Json
# The running game keeps the plugin it already loaded; replacing files is safe and
# takes effect on the next launch.
if (Get-Process -Name mhgurecomp -ErrorAction SilentlyContinue) { Write-Output '게임 실행 중: 다음 실행부터 적용됩니다.' }
foreach ($entry in $manifest.PSObject.Properties) {
    $hash = (Get-FileHash -LiteralPath (Join-Path $source $entry.Name) -Algorithm SHA256).Hash
    if ($hash -ne $entry.Value) { throw "배포 파일 해시 불일치: $($entry.Name)" }
}
$mods = [IO.Path]::GetFullPath((Join-Path $GameRoot 'mods'))
$destination = Join-Path $mods 'Better-Skillinfo'
$legacy = Join-Path $mods 'Better_Skillinfo'
if (Test-Path -LiteralPath $legacy) {
    if (Test-Path -LiteralPath $destination) { throw 'Better-Skillinfo와 Better_Skillinfo 폴더가 모두 존재합니다. 중복 설치를 정리하세요.' }
    if ((Split-Path -Parent ([IO.Path]::GetFullPath($legacy))) -ne $mods -or (Split-Path -Parent ([IO.Path]::GetFullPath($destination))) -ne $mods) { throw '모드 폴더 경로 검증 실패' }
    Move-Item -LiteralPath $legacy -Destination $destination
}
foreach ($entry in $manifest.PSObject.Properties) {
    $target = Join-Path $destination $entry.Name
    New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $source $entry.Name) -Destination $target -Force
    if ((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash -ne $entry.Value) { throw "설치 검증 실패: $target" }
}
# The user's load order stays as it is. Only the old folder name is renamed in place
# (keeping its position and on/off state). An unlisted folder loads after the listed
# ones, so a first install needs no line at all.
$order = Join-Path $GameRoot 'mods/load_order.txt'
if (Test-Path -LiteralPath $order) {
    $utf8 = New-Object System.Text.UTF8Encoding($false)
    $lines = [IO.File]::ReadAllLines($order, $utf8)
    $current = @($lines | Where-Object { $_ -in @('Better-Skillinfo','# off: Better-Skillinfo') }).Count -gt 0
    $changed = $false
    $result = foreach ($line in $lines) {
        if ($line -in @('Better_Skillinfo','# off: Better_Skillinfo')) {
            $changed = $true
            if (-not $current) { $current = $true; $line -replace 'Better_Skillinfo$','Better-Skillinfo' }
        } else { $line }
    }
    if ($changed) { [IO.File]::WriteAllLines($order, [string[]]@($result), $utf8) }
}
Write-Output "설치 및 해시 검증 완료: $destination"
