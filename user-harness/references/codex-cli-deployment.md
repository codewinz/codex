# Codex CLI Deployment Reference

Use this reference for machine-local Codewinz builds of the Codex CLI.

## Destination

Deploy to:

```text
C:\Users\codewinz\AppData\Local\Programs\OpenAI\Codex\bin
```

The deploy script defaults to the equivalent `$env:LOCALAPPDATA\Programs\OpenAI\Codex\bin` path.

## File Naming

Each deployment must create a new versioned executable:

```text
codex-codewinz-{baseVersion}+build.{N}.exe
```

`{baseVersion}` is normally read from the source executable's `--version` output, for example
`codex-cli 0.137.0`.

`{N}` is the build number. For the same `{baseVersion}`, increase it on every deployment by
scanning the install folder for existing matching files and using the next integer.

Example sequence:

```text
codex-codewinz-0.137.0+build.1.exe
codex-codewinz-0.137.0+build.2.exe
codex-codewinz-0.137.0+build.3.exe
```

## Latest Link

After copying the versioned executable, set:

```text
codex-codewinz.exe
```

as a `SymbolicLink` to the newest `codex-codewinz-{baseVersion}+build.{N}.exe`.

Do not update or replace unrelated executables such as `codex.exe`, `codex-winfix*.exe`, or
`codex-backup.exe` as part of this Codewinz deployment rule.
