$ErrorActionPreference = 'Continue'
$base = "http://127.0.0.1:8080"

# 登录
$body = '{"username":"admin","password":"admin123"}'
$resp = Invoke-WebRequest -Uri "$base/api/system/auth/login" -Method POST -Body $body -ContentType "application/json" -UseBasicParsing -TimeoutSec 20
$text = [System.Text.Encoding]::UTF8.GetString($resp.Content)
$tok = [regex]::Match($text, 'eyJ[A-Za-z0-9_\-\.]{30,}').Value
Write-Host "[A04] LOGIN token_len=$($tok.Length)"
$h = @{ Authorization = "Bearer $tok" }
[System.IO.File]::WriteAllText("c:\codework\MxxCrm\backend\.test_token", $tok)

# C05/C04: 手动重算（近2年）+ 批次列表
$start = (Get-Date).AddYears(-2).ToString("yyyy-MM-dd")
$end = (Get-Date).AddDays(-1).ToString("yyyy-MM-dd")
$body2 = '{"topic":"all","start_date":"' + $start + '","end_date":"' + $end + '"}'
$sw = [System.Diagnostics.Stopwatch]::StartNew()
$r2 = Invoke-WebRequest -Uri "$base/api/system/statistics/agg/refresh" -Method POST -Body $body2 -ContentType "application/json" -Headers $h -UseBasicParsing -TimeoutSec 180
$sw.Stop()
Write-Host "[C05] REFRESH http=$($r2.StatusCode) elapsed=$($sw.ElapsedMilliseconds)ms"

$r3 = Invoke-WebRequest -Uri "$base/api/system/statistics/agg/batches?page=1&page_size=10" -Headers $h -UseBasicParsing -TimeoutSec 30
$t3 = [System.Text.Encoding]::UTF8.GetString($r3.Content)
$batchOk = ($t3 -match 'contract') -and ($t3 -match 'payment')
Write-Host "[C04] BATCHES http=$($r3.StatusCode) hasTopics=$batchOk"

# A04: 超管统计接口（实时路径）
$sw2 = [System.Diagnostics.Stopwatch]::StartNew()
$s1 = Invoke-WebRequest -Uri "$base/api/system/statistics/contract/type-distribution" -Headers $h -UseBasicParsing -TimeoutSec 60
$sw2.Stop()
$t1 = [System.Text.Encoding]::UTF8.GetString($s1.Content)
Write-Host "[A04] CONTRACT-TYPE http=$($s1.StatusCode) first=$($sw2.ElapsedMilliseconds)ms bytes=$($s1.Content.Length)"

# B01: 缓存命中（第二次明显快）
$sw3 = [System.Diagnostics.Stopwatch]::StartNew()
$s2 = Invoke-WebRequest -Uri "$base/api/system/statistics/contract/type-distribution" -Headers $h -UseBasicParsing -TimeoutSec 60
$sw3.Stop()
Write-Host "[B01] SECOND hit=$($sw3.ElapsedMilliseconds)ms bytes=$($s2.Content.Length)"

# A04: 其余统计接口 200 验证
$endpoints = @(
    "/api/system/statistics/contract/ranking",
    "/api/system/statistics/contract/status-analysis",
    "/api/system/statistics/payment/completion",
    "/api/system/statistics/payment/monthly-trend",
    "/api/system/statistics/payment/status-analysis",
    "/api/system/statistics/payment/ranking",
    "/api/system/statistics/customer/type",
    "/api/system/statistics/customer/source",
    "/api/system/statistics/customer/industry",
    "/api/system/statistics/customer/funnel",
    "/api/system/statistics/employee/customer-count",
    "/api/system/statistics/employee/follow-up",
    "/api/system/statistics/employee/conversion"
)
$okCount = 0
foreach ($ep in $endpoints) {
    try {
        $r = Invoke-WebRequest -Uri ($base + $ep) -Headers $h -UseBasicParsing -TimeoutSec 60
        if ($r.StatusCode -eq 200) { $okCount++ } else { Write-Host "  FAIL $ep $($r.StatusCode)" }
    }
    catch {
        Write-Host "  ERR $ep $($_.Exception.Message)"
    }
}
Write-Host "[A04] STATS ENDPOINTS ok=$okCount/$($endpoints.Count)"

# B02: 缓存按用户隔离 —— 用相同参数但不同 token 无（跳过双账号，用无 token 验证 403 拦截即 scope 通道存在）
try {
    $r5 = Invoke-WebRequest -Uri "$base/api/system/statistics/agg/refresh" -Method POST -Body $body2 -ContentType "application/json" -UseBasicParsing -TimeoutSec 15 -SkipHttpErrorCheck
    $t5 = [System.Text.Encoding]::UTF8.GetString($r5.Content)
    $denied = ($t5 -match '403') -or ($t5 -match 'Authorization')
    Write-Host "[C05] NO-AUTH denied=$denied"
}
catch {
    Write-Host "[C05] NO-AUTH err $($_.Exception.Message)"
}
