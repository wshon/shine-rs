# MP3编码器测试套件运行脚本
# 这个脚本演示了完整的测试数据收集和验证流程

Write-Host "=== MP3编码器测试套件 ===" -ForegroundColor Green
Write-Host ""

# 检查必要文件是否存在
$testFiles = @("test_input.wav", "tests\input\sample-3s.wav")
foreach ($file in $testFiles) {
    if (-not (Test-Path $file)) {
        Write-Host "错误: 找不到测试文件 $file" -ForegroundColor Red
        exit 1
    }
}

Write-Host "1. 编译项目..." -ForegroundColor Yellow
cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Host "编译失败!" -ForegroundColor Red
    exit 1
}
Write-Host "✓ 编译成功" -ForegroundColor Green
Write-Host ""

Write-Host "2. 收集测试数据..." -ForegroundColor Yellow

# 收集基础测试用例数据
Write-Host "  收集 test_input.wav (128kbps)..."
cargo run --bin collect_test_data -- test_input.wav test_data_128k.json 128
if ($LASTEXITCODE -ne 0) {
    Write-Host "数据收集失败!" -ForegroundColor Red
    exit 1
}

# 收集不同比特率的测试数据
Write-Host "  收集 test_input.wav (192kbps)..."
cargo run --bin collect_test_data -- test_input.wav test_data_192k.json 192
if ($LASTEXITCODE -ne 0) {
    Write-Host "数据收集失败!" -ForegroundColor Red
    exit 1
}

# 收集长音频测试数据
Write-Host "  收集 sample-3s.wav (128kbps)..."
cargo run --bin collect_test_data -- tests\input\sample-3s.wav sample_3s_128k.json 128
if ($LASTEXITCODE -ne 0) {
    Write-Host "数据收集失败!" -ForegroundColor Red
    exit 1
}

Write-Host "✓ 测试数据收集完成" -ForegroundColor Green
Write-Host ""

Write-Host "3. 验证测试数据..." -ForegroundColor Yellow

$testCases = @(
    "test_data_128k.json",
    "test_data_192k.json", 
    "sample_3s_128k.json"
)

$passedTests = 0
$totalTests = $testCases.Length

foreach ($testCase in $testCases) {
    Write-Host "  验证 $testCase..."
    cargo run --bin validate_test_data -- $testCase
    if ($LASTEXITCODE -eq 0) {
        Write-Host "    ✓ 验证通过" -ForegroundColor Green
        $passedTests++
    } else {
        Write-Host "    ❌ 验证失败" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "=== 测试结果汇总 ===" -ForegroundColor Green
Write-Host "通过: $passedTests / $totalTests" -ForegroundColor $(if ($passedTests -eq $totalTests) { "Green" } else { "Yellow" })

if ($passedTests -eq $totalTests) {
    Write-Host "🎉 所有测试通过!" -ForegroundColor Green
    Write-Host ""
    Write-Host "生成的测试文件:" -ForegroundColor Cyan
    foreach ($testCase in $testCases) {
        if (Test-Path $testCase) {
            $size = (Get-Item $testCase).Length
            Write-Host "  - $testCase ($size 字节)" -ForegroundColor Gray
        }
    }
} else {
    Write-Host "💥 部分测试失败，请检查输出!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "测试套件运行完成!" -ForegroundColor Green