param(
  [Parameter(Mandatory = $true, ParameterSetName = 'Source')]
  [string] $SourceExe,

  [Parameter(Mandatory = $true, ParameterSetName = 'Build')]
  [switch] $Build,

  [string] $BaseVersion,

  [string] $InstallBin = (Join-Path $env:LOCALAPPDATA 'Programs\OpenAI\Codex\bin'),

  [switch] $Force
)

$ErrorActionPreference = 'Stop'

function Resolve-RequiredFile([string] $Path) {
  if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
    throw "Required file does not exist: $Path"
  }

  (Resolve-Path -LiteralPath $Path).Path
}

function Invoke-CodexVersion([string] $ExePath) {
  $output = & $ExePath --version 2>&1
  if ($LASTEXITCODE -ne 0) {
    throw "Version command failed for $ExePath with exit code $LASTEXITCODE`: $output"
  }

  ($output -join "`n").Trim()
}

function Get-BaseVersion([string] $ExePath, [string] $ExplicitBaseVersion) {
  if (-not [string]::IsNullOrWhiteSpace($ExplicitBaseVersion)) {
    return $ExplicitBaseVersion.Trim()
  }

  $versionOutput = Invoke-CodexVersion $ExePath
  if ($versionOutput -notmatch '^codex-cli\s+(.+)$') {
    throw "Could not parse Codex CLI version from: $versionOutput"
  }

  return ($Matches[1].Trim() -split '\+')[0]
}

function Assert-ValidBaseVersion([string] $Version) {
  if ($Version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$') {
    throw "BaseVersion must be SemVer without build metadata, got: $Version"
  }
}

function Invoke-CodewinzBuild {
  $repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).Path
  $codexRsRoot = Join-Path $repoRoot 'codex-rs'
  if (-not (Test-Path -LiteralPath (Join-Path $codexRsRoot 'Cargo.toml') -PathType Leaf)) {
    throw "Could not locate codex-rs Cargo.toml under: $codexRsRoot"
  }

  Push-Location $codexRsRoot
  try {
    cargo build --locked -p codex-cli --bin codex --profile codewinz-deploy
    if ($LASTEXITCODE -ne 0) {
      throw "Codewinz deploy build failed with exit code $LASTEXITCODE"
    }
  } finally {
    Pop-Location
  }

  $builtExe = Join-Path $codexRsRoot 'target\codewinz-deploy\codex.exe'
  Resolve-RequiredFile $builtExe
}

function Get-NextBuildNumber([string] $InstallDirectory, [string] $Version) {
  if (-not (Test-Path -LiteralPath $InstallDirectory -PathType Container)) {
    return 1
  }

  $escapedVersion = [regex]::Escape($Version)
  $pattern = "^codex-codewinz-$escapedVersion\+build\.([0-9]+)\.exe$"
  $maxBuild = 0

  foreach ($file in Get-ChildItem -LiteralPath $InstallDirectory -File -Filter 'codex-codewinz-*.exe') {
    if ($file.Name -match $pattern) {
      $build = [int] $Matches[1]
      if ($build -gt $maxBuild) {
        $maxBuild = $build
      }
    }
  }

  $maxBuild + 1
}

function Remove-ExistingLink([string] $Path, [switch] $AllowForce) {
  if (-not (Test-Path -LiteralPath $Path)) {
    return
  }

  $item = Get-Item -LiteralPath $Path -Force
  if ($item.LinkType -eq 'SymbolicLink') {
    Remove-Item -LiteralPath $Path -Force
    return
  }

  if (-not $AllowForce) {
    throw "Refusing to replace non-symbolic-link path without -Force: $Path"
  }

  Remove-Item -LiteralPath $Path -Force
}

$sourcePath = if ($Build) {
  Invoke-CodewinzBuild
} else {
  Resolve-RequiredFile $SourceExe
}
$installDirectory = $InstallBin
$base = Get-BaseVersion $sourcePath $BaseVersion
Assert-ValidBaseVersion $base

New-Item -ItemType Directory -Path $installDirectory -Force | Out-Null
$resolvedInstallDirectory = (Resolve-Path -LiteralPath $installDirectory).Path

$sourceVersionOutput = Invoke-CodexVersion $sourcePath
$buildNumber = Get-NextBuildNumber $resolvedInstallDirectory $base
$versionedName = "codex-codewinz-$base+build.$buildNumber.exe"
$versionedPath = Join-Path $resolvedInstallDirectory $versionedName
$linkPath = Join-Path $resolvedInstallDirectory 'codex-codewinz.exe'

if (Test-Path -LiteralPath $versionedPath) {
  if (-not $Force) {
    throw "Refusing to overwrite existing versioned executable without -Force: $versionedPath"
  }

  Remove-Item -LiteralPath $versionedPath -Force
}

Copy-Item -LiteralPath $sourcePath -Destination $versionedPath
Remove-ExistingLink $linkPath -AllowForce:$Force
New-Item -ItemType SymbolicLink -Path $linkPath -Target $versionedPath | Out-Null

$versionedOutput = Invoke-CodexVersion $versionedPath
$linkOutput = Invoke-CodexVersion $linkPath

if ($versionedOutput -ne $sourceVersionOutput) {
  throw "Versioned executable output differs from source. Source: $sourceVersionOutput Versioned: $versionedOutput"
}

if ($linkOutput -ne $sourceVersionOutput) {
  throw "Link executable output differs from source. Source: $sourceVersionOutput Link: $linkOutput"
}

[pscustomobject]@{
  SourceExe = $sourcePath
  SourceVersion = $sourceVersionOutput
  InstallBin = $resolvedInstallDirectory
  BaseVersion = $base
  BuildNumber = $buildNumber
  VersionedExe = $versionedPath
  Link = $linkPath
  LinkTarget = (Get-Item -LiteralPath $linkPath -Force).Target
} | Format-List
