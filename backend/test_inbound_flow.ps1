$base='http://127.0.0.1:8080/api/system'
$dir='C:\codework\MxxCrm\backend'

function Write-JsonFile($name, $obj) {
  $json = $obj | ConvertTo-Json -Depth 6 -Compress
  [System.IO.File]::WriteAllText("$dir\$name", $json, (New-Object System.Text.UTF8Encoding $false))
}

# 后端响应默认 MessagePack：curl -o 写二进制，python msgpack 解码为 JSON 字符串
function Curl-Decode([string[]]$argsArr) {
  $tmp = "$dir\resp_tmp.bin"
  Remove-Item $tmp -ErrorAction SilentlyContinue
  $fixed = @($argsArr) + @('-o', $tmp)
  & curl.exe -s @fixed *> $null
  if (-not (Test-Path $tmp)) { return '' }
  $json = & python "$dir\mp_decode.py" $tmp 2>$null
  if ($LASTEXITCODE -ne 0) { return '' }
  return ($json -join '')
}

function Get-Token($u,$p) {
  $body = '{"username":"' + $u + '","password":"' + $p + '"}'
  [System.IO.File]::WriteAllText("$dir\login_body.json", $body, (New-Object System.Text.UTF8Encoding $false))
  $all = Curl-Decode @('-X','POST','-H','Content-Type: application/json; charset=utf-8','--data-binary',"@$dir\login_body.json", "$base/auth/login")
  try {
    $j = $all | ConvertFrom-Json
    return $j.data.accessToken
  } catch { return '' }
}

function Post-Api($url, $token, $bodyObj) {
  Write-JsonFile 'req_body.json' $bodyObj
  $h = @('-X','POST','-H','Content-Type: application/json; charset=utf-8')
  if ($token) { $h += @('-H',"Authorization: Bearer $token") }
  $h += @('--data-binary',"@$dir\req_body.json", "$base$url")
  return Curl-Decode $h
}

$opToken = Get-Token 'operator' 'admin123'
$whToken = Get-Token 'warehouse' 'admin123'
$ceoToken = Get-Token 'ceo' 'admin123'
Write-Output "=== STEP0 tokens: op=$($opToken.Length) wh=$($whToken.Length) ceo=$($ceoToken.Length) ==="

# STEP1 operator create inbound (status=0)
$create = @{
  inboundType='purchase'; warehouseId=1; totalQuantity=4; totalAmount=23000.00; remark='operator test inbound';
  items=@(
    @{productId=1;quantity=3;unitPrice=5000;amount=15000.00},
    @{productId=2;quantity=1;unitPrice=8000;amount=8000.00}
  )
}
$createStr = Post-Api '/inbound/save' $opToken $create
Write-Output "=== STEP1 create resp: $createStr"

# fetch latest inbound id created by operator (id=15) from db
$env:PGPASSWORD='jyXyh2618BjSkf'
$inboundId = (psql -h 115.190.210.106 -p 5432 -U postgres -d mxxcrm_data -t -A -c "SELECT id FROM mxx_inventory_inbound WHERE created_by=15 ORDER BY id DESC LIMIT 1;").Trim()
Write-Output "=== STEP1 inbound id=$inboundId ==="

# STEP2 operator submit (status 0 -> 1, creates approval instance)
$submitStr = Curl-Decode @('-X','PUT','-H',"Authorization: Bearer $opToken", "$base/inbound/submit/$inboundId")
Write-Output "=== STEP2 operator submit resp: $submitStr"

# STEP3 get_detail must return instance (candidateApprovers contains warehouse 14)
$infoStr = Curl-Decode @('-H',"Authorization: Bearer $opToken", "$base/inbound/info?id=$inboundId")
Write-Output "=== STEP3 detail(has instance?): $([regex]::IsMatch($infoStr, '"instance":\s*\{'))"

# extract instance id
$instId = $null
try {
  $infoJson = $infoStr | ConvertFrom-Json
  $instId = $infoJson.data.instance.id
  Write-Output "=== STEP3 instanceId=$instId status=$($infoJson.data.detail.status) candidateApprovers=$($infoJson.data.instance.candidateApprovers -join ',') currentApprover=$($infoJson.data.instance.currentApproverName)"
} catch {
  Write-Output "=== STEP3 parse error: $_"
}

# STEP4 operator (submitter) cc to CEO(13) - "抄送给老板" scenario
$ccStr = Post-Api '/approval/cc/add' $opToken @{instanceId=[int64]$instId; userIds=@([int64]13); ccReason='请老板知悉本次采购入库'}
Write-Output "=== STEP4 operator add-cc resp: $ccStr"

# STEP5 CEO sees the cc (cc list) - params are page / pageSize
$ccListStr = Curl-Decode @('-H',"Authorization: Bearer $ceoToken", "$base/approval/cc/list?page=1&pageSize=20")
Write-Output "=== STEP5 ceo cc-list resp: $ccListStr"

# STEP6 warehouse add-sign 后加签(type=2) to CEO(13)
$addSignStr = Post-Api '/approval/add-sign' $whToken @{instanceId=[int64]$instId; addSignType=2; targetUserIds=@([int64]13); comment='加签CEO知悉'}
Write-Output "=== STEP6 warehouse add-sign resp: $addSignStr"

# STEP7 warehouse audit pass (status 1 -> 3, inventory updated)
$auditStr = Post-Api '/inbound/audit' $whToken @{id=[int64]$inboundId; comment='审核通过'}
Write-Output "=== STEP7 warehouse audit resp: $auditStr"

# STEP8 final detail: status should be 3, instance.status=3, ccUsers contains CEO
$finStr = Curl-Decode @('-H',"Authorization: Bearer $opToken", "$base/inbound/info?id=$inboundId")
try {
  $finJson = $finStr | ConvertFrom-Json
  $inst = $finJson.data.instance
  $ccNames = @($inst.ccUsers | ForEach-Object { $_.userName })
  Write-Output "=== STEP8 final: status=$($finJson.data.detail.status) instance.status=$($inst.status) ccUsers=$($ccNames -join ',') logs=$($inst.logs.Count)"
} catch {
  Write-Output "=== STEP8 parse error: $_"
}
Write-Output "=== STEP8 raw: $finStr"

# STEP9 verify inventory stock rows for warehouse 1 / product 1,2
$stockRaw = psql -h 115.190.210.106 -p 5432 -U postgres -d mxxcrm_data -t -A -c "SELECT product_id, quantity FROM mxx_inventory_stock WHERE warehouse_id=1 AND product_id IN (1,2) ORDER BY product_id;"
Write-Output "=== STEP9 stock: $stockRaw"
