# DevIgnite Full Verification Script (PowerShell)
Write-Host "===> Running Rust Backend Unit Tests..." -ForegroundColor Cyan
cargo test --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) {
    Write-Error "Rust tests failed!"
    exit 1
}

Write-Host "===> Running TypeScript Type Checks..." -ForegroundColor Cyan
npx tsc --noEmit
if ($LASTEXITCODE -ne 0) {
    Write-Error "TypeScript check failed!"
    exit 1
}

Write-Host "===> Running Vite Production Build..." -ForegroundColor Cyan
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Error "Vite build failed!"
    exit 1
}

Write-Host "===> All checks passed cleanly!" -ForegroundColor Green
