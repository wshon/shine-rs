# Publish script for the crate directory
param(
    [string]$Version = "",
    [switch]$DryRun = $false
)

Write-Host "🚀 Publishing shine-rs from crate/ directory" -ForegroundColor Green

# Change to crate directory
Push-Location "crate"

try {
    # Check if version is provided for non-dry-run
    if (-not $DryRun -and $Version -eq "") {
        Write-Host "❌ Please provide version number: .\scripts\publish_crate.ps1 -Version '0.1.0'" -ForegroundColor Red
        exit 1
    }

    # Update version if provided
    if ($Version -ne "") {
        Write-Host "📝 Updating version to $Version..." -ForegroundColor Yellow
        (Get-Content Cargo.toml) -replace 'version = ".*"', "version = `"$Version`"" | Set-Content Cargo.toml
    }

    # Check compilation
    Write-Host "🔍 Checking compilation..." -ForegroundColor Yellow
    cargo check
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Compilation failed!" -ForegroundColor Red
        exit 1
    }

    # Run tests
    Write-Host "🧪 Running tests..." -ForegroundColor Yellow
    cargo test --lib
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Tests failed!" -ForegroundColor Red
        exit 1
    }

    if ($DryRun) {
        Write-Host "🔍 Performing dry run..." -ForegroundColor Yellow
        cargo publish --dry-run --registry crates-io
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Dry run successful! Ready to publish." -ForegroundColor Green
        } else {
            Write-Host "❌ Dry run failed!" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "🚀 Publishing to crates.io..." -ForegroundColor Yellow
        cargo publish --registry crates-io
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "🎉 Successfully published shine-rs v$Version!" -ForegroundColor Green
            Write-Host "📦 Check your package at: https://crates.io/crates/shine-rs" -ForegroundColor Cyan
            Write-Host "📚 Documentation will be available at: https://docs.rs/shine-rs" -ForegroundColor Cyan
            
            # Update version in root Cargo.toml as well
            Pop-Location
            Write-Host "📝 Updating root Cargo.toml version..." -ForegroundColor Yellow
            (Get-Content Cargo.toml) -replace 'version = ".*"', "version = `"$Version`"" | Set-Content Cargo.toml
            
            # Commit and tag
            Write-Host "📦 Committing and tagging version $Version..." -ForegroundColor Yellow
            git add .
            git commit -m "Release v$Version"
            git tag "v$Version"
            git push origin main --tags
        } else {
            Write-Host "❌ Publishing failed!" -ForegroundColor Red
            exit 1
        }
    }
} finally {
    Pop-Location
}

Write-Host "✨ Done!" -ForegroundColor Green