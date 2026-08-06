# PDF Acceptance Test Script - Login inside
$ErrorActionPreference = "Continue"
$BASE = "http://127.0.0.1:8080/api/system"
$results = @()

# Step 1: Login to get fresh token
Write-Host "Logging in..." -ForegroundColor Yellow
$body = '{"username":"admin","password":"admin123","captchaId":"","captchaCode":""}'
$loginResp = Invoke-WebRequest -Uri "$BASE/auth/login" -Method Post -Body $body -ContentType "application/json" -UseBasicParsing
$loginBytes = $loginResp.RawContentStream.ToArray()
$loginText = [System.Text.Encoding]::UTF8.GetString($loginBytes)
$tokenMatch = [regex]::Match($loginText, 'eyJ[A-Za-z0-9_\-]+\.eyJ[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+')
if (-not $tokenMatch.Success) {
    Write-Host "FATAL: Login failed, no token found" -ForegroundColor Red
    exit 1
}
$TOKEN = $tokenMatch.Value
Write-Host "Token obtained, length=$($TOKEN.Length)" -ForegroundColor Green
$H = @{ Authorization = "Bearer $TOKEN" }

function Test-Api($name, $method, $url, $body=$null, $expectSuccess=$true) {
    try {
        $params = @{ Uri = $url; Method = $method; Headers = $H; UseBasicParsing = $true }
        if ($body) {
            $params.Body = ($body | ConvertTo-Json -Depth 10)
            $params.ContentType = "application/json"
        }
        $r = Invoke-WebRequest @params
        $bytes = $r.RawContentStream.ToArray()
        $text = [System.Text.Encoding]::UTF8.GetString($bytes)
        $isOk = ($text -match '"code"\s*:\s*0') -or ($text -match '"code"\s*:\s*200') -or ($r.StatusCode -eq 200 -and $bytes.Length -gt 0 -and $method -eq "GET")
        $status = if ($isOk) { "PASS" } else { "FAIL" }
        $info = "Status=$($r.StatusCode), Len=$($bytes.Length)"
        Write-Host "[$status] $name | $info" -ForegroundColor $(if ($isOk) {"Green"} else {"Red"})
        $script:results += [PSCustomObject]@{ Name=$name; Status=$status; Info=$info }
        return @{ Text=$text; Bytes=$bytes; IsOk=$isOk }
    } catch {
        $status = if (-not $expectSuccess) { "PASS-ERR" } else { "FAIL" }
        $info = "Error: $($_.Exception.Message)"
        Write-Host "[$status] $name | $info" -ForegroundColor $(if ($status -like "PASS*") {"Green"} else {"Red"})
        $script:results += [PSCustomObject]@{ Name=$name; Status=$status; Info=$info }
        return @{ Text=""; Bytes=@(); IsOk=$false; Error=$_.Exception.Message }
    }
}

Write-Host ""
Write-Host "========== 14.2.4 Template Management (T-01~T-07) ==========" -ForegroundColor Cyan
Test-Api "T-01 Template List" "GET" "$BASE/pdf-template/list?page=1&pageSize=20"
Test-Api "T-02 Template Info(id=1)" "GET" "$BASE/pdf-template/info?id=1"
Test-Api "T-07 Options(quotation)" "GET" "$BASE/pdf-template/options?docType=quotation"
Test-Api "T-07 Options(order)" "GET" "$BASE/pdf-template/options?docType=order"
Test-Api "T-07 Options(contract)" "GET" "$BASE/pdf-template/options?docType=contract"

$contentStr = "#align(center)[*Test Template*]"
$newTpl = @{
    name = "Test Quotation Template"
    templateCode = "test_quo_001"
    docType = "quotation"
    content = $contentStr
    paperSize = "a4"
    orientation = "portrait"
    marginTop = 20
    marginBottom = 20
    marginLeft = 40
    marginRight = 40
    fontFamily = "Source Han Sans SC"
    isDefault = 0
    status = 1
    sort = 99
    remark = "acceptance test"
}
$r = Test-Api "T-02 Create Template" "POST" "$BASE/pdf-template/save" $newTpl
$newId = if ($r.Text -match '\b(\d{1,15})\b') { $matches[1] } else { $null }
Write-Host "  New Template ID: $newId" -ForegroundColor Yellow

if ($newId -and $newId -ne "0") {
    $updContent = "#align(center)[*Test Template Modified*]"
    $updBody = @{
        id = [long]$newId
        name = "Test Quotation Template v2"
        templateCode = "test_quo_001"
        docType = "quotation"
        content = $updContent
        paperSize = "a4"
        orientation = "landscape"
        marginTop = 15
        marginBottom = 15
        marginLeft = 30
        marginRight = 30
        fontFamily = "Source Han Sans SC"
        isDefault = 0
        status = 1
        sort = 99
        remark = "modified"
    }
    Test-Api "T-03 Update Template(id=$newId)" "PUT" "$BASE/pdf-template/update" $updBody
    Test-Api "T-04 Set Default(id=$newId)" "PUT" "$BASE/pdf-template/set_default?id=$newId"
    $delBody = @{ ids = @([long]$newId) }
    Test-Api "T-05 Delete Template(id=$newId)" "POST" "$BASE/pdf-template/bath_delete" $delBody
}

Write-Host ""
Write-Host "========== 14.2.1 Quotation PDF (Q-01~Q-08) ==========" -ForegroundColor Cyan
$r = Test-Api "Q-02 Generate Quotation PDF(id=110)" "POST" "$BASE/pdf/generate" @{ docType="quotation"; docId=110 }
Test-Api "Q-06 History Records(quotation 110)" "GET" "$BASE/pdf/record-list?docType=quotation&docId=110&page=1&pageSize=10"

Write-Host ""
Write-Host "========== 14.2.2 Order PDF (O-01~O-03) ==========" -ForegroundColor Cyan
$r = Test-Api "O-01 Generate Order PDF(id=111)" "POST" "$BASE/pdf/generate" @{ docType="order"; docId=111 }

Write-Host ""
Write-Host "========== 14.2.3 Contract PDF (C-01~C-07) ==========" -ForegroundColor Cyan
$r = Test-Api "C-01 Generate Contract PDF(id=108)" "POST" "$BASE/pdf/generate" @{ docType="contract"; docId=108 }

Write-Host ""
Write-Host "========== 14.2.5 PDF Download (D-01~D-04) ==========" -ForegroundColor Cyan
$env:PGPASSWORD="123456"
$recIdRaw = & psql -h 127.0.0.1 -U postgres -d mxxcrm_data -t -c "SELECT id FROM mxx_system_pdf_record WHERE deleted=0 ORDER BY id DESC LIMIT 1;" 2>$null
$recId = $recIdRaw.Trim()
Write-Host "  Latest Record ID: $recId" -ForegroundColor Yellow
if ($recId) {
    Test-Api "D-01 Download PDF(id=$recId)" "GET" "$BASE/pdf/download?id=$recId"
}

Write-Host ""
Write-Host "========== 14.5 Error Handling (A-01~A-06) ==========" -ForegroundColor Cyan
$badContent = "#this_is_invalid_typst_syntax(((("
$badTpl = @{
    name = "Bad Template"
    templateCode = "test_bad_001"
    docType = "quotation"
    content = $badContent
    paperSize = "a4"
    orientation = "portrait"
    marginTop = 20
    marginBottom = 20
    marginLeft = 40
    marginRight = 40
    fontFamily = "Source Han Sans SC"
    isDefault = 0
    status = 1
    sort = 99
}
$r = Test-Api "A-01a Create Bad Template" "POST" "$BASE/pdf-template/save" $badTpl
$badId = if ($r.Text -match '\b(\d{1,15})\b') { $matches[1] } else { $null }
if ($badId -and $badId -ne "0") {
    Test-Api "A-01b Generate with Bad Template(should fail)" "POST" "$BASE/pdf/generate" @{ docType="quotation"; docId=110; templateId=[long]$badId } $false
    Test-Api "A-01c Cleanup Bad Template" "POST" "$BASE/pdf-template/bath_delete" @{ ids=@([long]$badId) }
}
Test-Api "A-03 Doc Not Exist(quotation 999999)" "POST" "$BASE/pdf/generate" @{ docType="quotation"; docId=999999 } $false
Test-Api "A-04 Template Not Exist(id=999999)" "GET" "$BASE/pdf-template/info?id=999999" $false

try {
    $r = Invoke-WebRequest -Uri "$BASE/pdf-template/list?page=1&pageSize=1" -UseBasicParsing
    Write-Host "[FAIL] D-02 No Token Should Be Rejected" -ForegroundColor Red
    $script:results += [PSCustomObject]@{ Name="D-02 No Token"; Status="FAIL"; Info="Should reject" }
} catch {
    Write-Host "[PASS] D-02 No Token Rejected Correctly" -ForegroundColor Green
    $script:results += [PSCustomObject]@{ Name="D-02 No Token"; Status="PASS-ERR"; Info="Rejected" }
}

Write-Host ""
Write-Host "========== Summary ==========" -ForegroundColor Cyan
$pass = ($results | Where-Object { $_.Status -like "PASS*" }).Count
$fail = ($results | Where-Object { $_.Status -eq "FAIL" }).Count
Write-Host "Total: $($results.Count)  Pass: $pass  Fail: $fail" -ForegroundColor $(if ($fail -eq 0) {"Green"} else {"Yellow"})
$results | Format-Table -AutoSize
