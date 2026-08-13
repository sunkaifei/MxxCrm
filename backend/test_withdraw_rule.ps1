$base='http://127.0.0.1:8080/api/system'
$dir='C:\codework\MxxCrm\backend'

function Write-JsonFile($name, $obj) {
  $json = $obj | ConvertTo-Json -Depth 6 -Compress
  [System.IO.File]::WriteAllText("$dir\$name", $json, (New-Object System.Text.UTF8Encoding $false))
}

function Get-Token($u,$p) {
  $body = '{"username":"' + $u + '","password":"' + $p + '"}'
  [System.IO.File]::WriteAllText("$dir\login_body.json", $body, (New-Object System.Text.UTF8Encoding $false))
  $raw = curl.exe -s -X POST -H "Content-Type: application/json; charset=utf-8" --data-binary "@$dir\login_body.json" "$base/auth/login"
  $all = $raw -join "`n"
  $m = [regex]::Match($all, 'eyJ[A-Za-z0-9_\-\.]+')
  return $m.Value
}

# 1. operator creates + submits a new inbound
$opToken = Get-Token 'operator' 'admin123'
$create = @{
  inboundType='purchase'; warehouseId=1; totalQuantity=2; totalAmount=10000.00; remark='withdraw-rule test';
  items=@(@{productId=1;quantity=2;unitPrice=5000;amount=10000.00})
}
Write-JsonFile 'create_body.json' $create
curl.exe -s -X POST -H "Content-Type: application/json; charset=utf-8" -H "Authorization: Bearer $opToken" --data-binary "@$dir\create_body.json" "$base/inbound/save" | Out-Null

$env:PGPASSWORD='jyXyh2618BjSkf'
$inboundId = (psql -h 115.190.210.106 -p 5432 -U postgres -d mxxcrm_data -t -A -c "SELECT id FROM mxx_inventory_inbound WHERE remark='withdraw-rule test' ORDER BY id DESC LIMIT 1;").Trim()
Write-Output "=== created id=$inboundId ==="

$submitRaw = curl.exe -s -X PUT -H "Authorization: Bearer $opToken" "$base/inbound/submit/$inboundId"
Write-Output "=== submit: $($submitRaw -join '')"

# 2. warehouse (not creator) tries to withdraw -> should FAIL
$whToken = Get-Token 'warehouse' 'admin123'
Write-JsonFile 'withdraw_body.json' @{id=[int64]$inboundId}
$wdRaw = curl.exe -s -X POST -H "Content-Type: application/json; charset=utf-8" -H "Authorization: Bearer $whToken" --data-binary "@$dir\withdraw_body.json" "$base/inbound/withdraw"
Write-Output "=== warehouse withdraw (expect fail): $($wdRaw -join '')"

# 3. operator (creator) withdraws -> should SUCCEED
$wd2Raw = curl.exe -s -X POST -H "Content-Type: application/json; charset=utf-8" -H "Authorization: Bearer $opToken" --data-binary "@$dir\withdraw_body.json" "$base/inbound/withdraw"
Write-Output "=== operator withdraw (expect success): $($wd2Raw -join '')"

$status = (psql -h 115.190.210.106 -p 5432 -U postgres -d mxxcrm_data -t -A -c "SELECT status FROM mxx_inventory_inbound WHERE id=$inboundId;").Trim()
Write-Output "=== final status=$status (0=draft expected) ==="
