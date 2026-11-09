# Signs the Windows build artifacts (exe, MSI, NSIS) with Authenticode + RFC3161 timestamp.
#
# Certificate selection:
#   1. $env:FAKESQL_SIGN_PFX + $env:FAKESQL_SIGN_PWD  -> real/production certificate (CI, CA cert, Azure Trusted Signing export)
#   2. Fallback: the local development certificate at %USERPROFILE%\.certs\fakesql-codesign.pfx
#      (self-signed, trusted only on this machine; password: fakesql-local)
#
# Without any certificate the script exits gracefully (artifacts stay unsigned).
# Windows: unsigned binaries trigger the SmartScreen / "editeur inconnu" warnings.

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

# --- collect artifacts -------------------------------------------------------
$artifacts = @(
  (Join-Path $root 'src-tauri\target\release\fakesql-studio.exe')
)
foreach ($dirName in @('msi', 'nsis')) {
  $dir = Join-Path $root ('src-tauri\target\release\bundle\' + $dirName)
  if (Test-Path $dir) {
    $artifacts += Get-ChildItem $dir -Include '*.msi', '*.exe' -Recurse | ForEach-Object { $_.FullName }
  }
}
$artifacts = $artifacts | Where-Object { Test-Path $_ } | Select-Object -Unique
if (-not $artifacts) {
  Write-Warning "Aucun artefact a signer. Lancez d'abord : npm run tauri build"
  exit 1
}

# --- locate signtool ---------------------------------------------------------
$signTool = Get-Command signtool.exe -ErrorAction SilentlyContinue
if (-not $signTool) {
  $cand = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin' -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
    Sort-Object FullName -Descending | Select-Object -First 1
  if ($cand) { $signTool = $cand.FullName }
}
if (-not $signTool) {
  Write-Warning "signtool.exe introuvable (Windows SDK requis). Artefact non signes."
  exit 1
}
$signTool = $signTool.Source

# --- select certificate ------------------------------------------------------
$pfx = $env:FAKESQL_SIGN_PFX
$pass = $env:FAKESQL_SIGN_PWD
if (-not $pfx) {
  $devPfx = Join-Path $env:USERPROFILE '.certs\fakesql-codesign.pfx'
  if (Test-Path $devPfx) {
    $pfx = $devPfx
    $pass = 'fakesql-local'
    Write-Host "Utilisation du certificat de developpement local : $pfx"
  }
}
if (-not $pfx) {
  Write-Warning "Aucun certificat disponible (FAKESQL_SIGN_PFX non defini, pas de certificat local). Artefacts laisses non signes."
  exit 0
}

# --- sign + timestamp --------------------------------------------------------
& $signTool sign /f $pfx /p $pass /fd SHA256 /td SHA256 /tr http://timestamp.digicert.com $artifacts
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# --- verify ------------------------------------------------------------------
foreach ($file in $artifacts) {
  $status = (Get-AuthenticodeSignature $file).Status.ToString()
  Write-Host ("{0} : {1}" -f (Split-Path $file -Leaf), $status)
  if ($status -ne 'Valid') { exit 1 }
}
Write-Host "Tous les artefacts sont signes."
