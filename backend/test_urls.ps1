$urls = @(
    "/",
    "/index.html",
    "/category/about",
    "/category/products",
    "/category/news",
    "/category/support",
    "/category/company",
    "/category/team",
    "/category/industry",
    "/article/welcome-to-mxxcrm",
    "/article/crm-best-practices",
    "/article/industry-trends-2026",
    "/article/about-us",
    "/article/service-support",
    "/product",
    "/product/",
    "/search?keyword=CRM",
    "/sitemap",
    "/sitemap.html",
    "/page/contact",
    "/page/privacy"
)
$base = "http://localhost:8080"
foreach ($u in $urls) {
    try {
        $r = Invoke-WebRequest -Uri ($base + $u) -UseBasicParsing -TimeoutSec 15
        Write-Host ("{0,3}  {1,7}  {2}" -f $r.StatusCode, $r.Content.Length, $u)
    } catch {
        $code = "ERR"
        if ($_.Exception.Response) { $code = [int]$_.Exception.Response.StatusCode }
        Write-Host ("{0,3}  {1,7}  {2}" -f $code, "-", $u)
    }
}
