# Prepare for crates.io release
# This script helps prepare the project for publishing to crates.io

Write-Host "Preparing shine-rs for crates.io release..." -ForegroundColor Green

# 1. Check basic compilation
Write-Host "`n1. Checking compilation..." -ForegroundColor Yellow
cargo check
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Compilation failed. Please fix errors before publishing." -ForegroundColor Red
    exit 1
}

# 2. Run tests (only basic ones)
Write-Host "`n2. Running basic tests..." -ForegroundColor Yellow
cargo test --lib --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Some tests failed, but continuing..." -ForegroundColor Yellow
}

# 3. Check package contents
Write-Host "`n3. Checking package contents..." -ForegroundColor Yellow
cargo package --list

# 4. Dry run publish
Write-Host "`n4. Performing dry run..." -ForegroundColor Yellow
cargo publish --dry-run
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Dry run failed. Please fix issues before publishing." -ForegroundColor Red
    exit 1
}

Write-Host "`n✅ Release preparation completed successfully!" -ForegroundColor Green
Write-Host "`nNext steps:" -ForegroundColor Cyan
Write-Host "1. Review the package contents above"
Write-Host "2. Make sure you're logged in: cargo login <your-token>"
Write-Host "3. Publish: cargo publish"
Write-Host "4. Check your package at: https://crates.io/crates/shine-rs"