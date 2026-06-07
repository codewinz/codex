param(
  [string] $Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
)

$ErrorActionPreference = 'Stop'

$requiredPaths = @(
  'INDEX.md',
  'WORKFLOW.md',
  'QUALITY.md',
  'PREFERENCES.md',
  'RELIABILITY.md',
  'PLANS.md',
  'MULTI_AGENT.md',
  'references',
  'references/testing.md',
  'references/sentry-tdd.md',
  'references/shared-worktree-context.md',
  'references/codex-cli-deployment.md',
  'scripts/deploy-codex-codewinz.ps1',
  'templates',
  'templates/execution-plan.md',
  'templates/bugfix-tdd.md',
  'templates/review-checklist.md'
)

$missing = @()
foreach ($relativePath in $requiredPaths) {
  $path = Join-Path $Root $relativePath
  if (-not (Test-Path -LiteralPath $path)) {
    $missing += $relativePath
  }
}

if ($missing.Count -gt 0) {
  throw "Missing required user-harness paths:`n$($missing -join "`n")"
}

function Read-HarnessFile([string] $RelativePath) {
  Get-Content -LiteralPath (Join-Path $Root $RelativePath) -Raw
}

$index = Read-HarnessFile 'INDEX.md'
foreach ($requiredLink in @(
  'WORKFLOW.md',
  'QUALITY.md',
  'PREFERENCES.md',
  'RELIABILITY.md',
  'PLANS.md',
  'MULTI_AGENT.md',
  'references/',
  'templates/'
)) {
  if ($index -notmatch [regex]::Escape($requiredLink)) {
    throw "INDEX.md is missing a link or reference to $requiredLink."
  }
}

if ($index -notmatch 'When project docs conflict with this harness') {
  throw 'INDEX.md is missing the project-doc precedence note.'
}

$multiAgent = Read-HarnessFile 'MULTI_AGENT.md'
foreach ($requiredToken in @('explorer', 'worker', 'reviewer', 'read-only', 'coordinator')) {
  if ($multiAgent -notmatch [regex]::Escape($requiredToken)) {
    throw "MULTI_AGENT.md is missing expected content: $requiredToken"
  }
}

$sharedContext = Read-HarnessFile 'references/shared-worktree-context.md'
foreach ($requiredToken in @('<git-common-dir>/agent-context/', 'sanitized tracked exports', 'Never store secrets')) {
  if ($sharedContext -notmatch [regex]::Escape($requiredToken)) {
    throw "shared-worktree-context.md is missing expected content: $requiredToken"
  }
}

$deployment = Read-HarnessFile 'references/codex-cli-deployment.md'
foreach ($requiredToken in @(
  'C:\Users\codewinz\AppData\Local\Programs\OpenAI\Codex\bin',
  'codewinz-deploy',
  'deploy-codex-codewinz.ps1 -Build',
  'codex-codewinz-{baseVersion}+build.{N}.exe',
  'codex-codewinz.exe',
  'build number',
  'SymbolicLink'
)) {
  if ($deployment -notmatch [regex]::Escape($requiredToken)) {
    throw "codex-cli-deployment.md is missing expected content: $requiredToken"
  }
}

$deployScript = Read-HarnessFile 'scripts/deploy-codex-codewinz.ps1'
foreach ($requiredToken in @(
  'SourceExe',
  'Build',
  'BaseVersion',
  'InstallBin',
  'cargo build --locked -p codex-cli --bin codex --profile codewinz-deploy',
  'target\codewinz-deploy\codex.exe',
  'codex-codewinz-$base+build.$buildNumber.exe',
  'Get-NextBuildNumber',
  'New-Item -ItemType SymbolicLink'
)) {
  if ($deployScript -notmatch [regex]::Escape($requiredToken)) {
    throw "deploy-codex-codewinz.ps1 is missing expected content: $requiredToken"
  }
}

$markdownFiles = Get-ChildItem -LiteralPath $Root -Recurse -Filter '*.md'
$repoSpecificPatterns = @(
  'D:\\Repositories\\codewinz\\golden-goose',
  '@golden-goose',
  'deploy:production',
  'deploy:ubuntu-stack',
  'npm run harness:',
  'KIS',
  'broker credential',
  'trading software'
)

foreach ($file in $markdownFiles) {
  $content = Get-Content -LiteralPath $file.FullName -Raw
  foreach ($pattern in $repoSpecificPatterns) {
    if ($content -match [regex]::Escape($pattern)) {
      $relativePath = Resolve-Path -LiteralPath $file.FullName -Relative
      throw "Repo-specific Golden Goose content found in $relativePath`: $pattern"
    }
  }
}

Write-Output 'User harness validation passed.'
