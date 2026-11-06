<#
.SYNOPSIS
  Regenerates tests/scripts/*.out and *.code from real Git Bash.

.DESCRIPTION
  For every tests/scripts/*.sh (optionally filtered by a substring of the file
  name) this runs
      "C:\Program Files\Git\bin\bash.exe" NNN_topic.sh
  with the working directory set to tests/scripts, stdin from NUL, stdout
  captured byte-exact (via cmd.exe redirection, so PowerShell never re-encodes
  it) and stderr captured for diagnostics only.

  Each script is run twice; if stdout bytes or the exit code differ between the
  two runs the script is reported as FLAKY and its expected files are not
  written.

  The child environment matches tests/compat.rs: LC_ALL=C is set and LANG,
  LC_*, BASH_ENV, ENV, SHELLOPTS, BASHOPTS, CDPATH, GLOBIGNORE and
  POSIXLY_CORRECT are removed, so glob/sort collation does not depend on the
  terminal the tests are run from.

  .sh files are normalized to LF line endings before running.

.PARAMETER Filter
  Optional substring; only scripts whose file name contains it are processed.

.PARAMETER Bash
  Path to Git Bash (default: C:\Program Files\Git\bin\bash.exe).

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File tests\regen_expected.ps1
  powershell -ExecutionPolicy Bypass -File tests\regen_expected.ps1 heredoc
#>
param(
    [string]$Filter = "",
    [string]$Bash = "C:\Program Files\Git\bin\bash.exe"
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 2

$scriptsDir = Join-Path $PSScriptRoot 'scripts'
if (-not (Test-Path -LiteralPath $Bash)) { throw "Git Bash not found at $Bash" }
if (-not (Test-Path -LiteralPath $scriptsDir)) { throw "scripts dir not found: $scriptsDir" }

$tmpDir = Join-Path ([IO.Path]::GetTempPath()) ("gash_regen_" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmpDir | Out-Null

$removeExact = @('LANG', 'BASH_ENV', 'ENV', 'SHELLOPTS', 'BASHOPTS', 'CDPATH', 'GLOBIGNORE', 'POSIXLY_CORRECT')
$timeoutMs = 60000

function Invoke-GitBash([string]$name, [int]$run) {
    $outFile = Join-Path $tmpDir "$name.$run.stdout"
    $errFile = Join-Path $tmpDir "$name.$run.stderr"
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = Join-Path $env:SystemRoot 'System32\cmd.exe'
    # /s: strip only the outermost quotes and keep the rest verbatim.
    $psi.Arguments = "/d /s /c `" `"$Bash`" `"$name`" <NUL >`"$outFile`" 2>`"$errFile`" `""
    $psi.WorkingDirectory = $scriptsDir
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $envVars = $psi.EnvironmentVariables
    foreach ($k in @($envVars.Keys)) {
        if ($removeExact -contains $k.ToUpperInvariant() -or $k.ToUpperInvariant().StartsWith('LC_')) {
            $envVars.Remove($k)
        }
    }
    $envVars['LC_ALL'] = 'C'
    $p = [System.Diagnostics.Process]::Start($psi)
    if (-not $p.WaitForExit($timeoutMs)) {
        try { $p.Kill() } catch { }
        return [pscustomobject]@{ Stdout = [byte[]]@(); Stderr = "TIMEOUT"; Code = -1; TimedOut = $true }
    }
    $p.WaitForExit()
    $stdout = if (Test-Path -LiteralPath $outFile) { [IO.File]::ReadAllBytes($outFile) } else { [byte[]]@() }
    $stderr = if (Test-Path -LiteralPath $errFile) { [IO.File]::ReadAllText($errFile) } else { "" }
    return [pscustomobject]@{ Stdout = $stdout; Stderr = $stderr; Code = $p.ExitCode; TimedOut = $false }
}

$scripts = Get-ChildItem -LiteralPath $scriptsDir -Filter '*.sh' | Sort-Object Name
if ($Filter) { $scripts = @($scripts | Where-Object { $_.Name.Contains($Filter) }) }
if (@($scripts).Count -eq 0) { Write-Host "no scripts match filter '$Filter'"; exit 1 }

$written = 0
$problems = @()
try {
    foreach ($s in $scripts) {
        $name = $s.Name
        $base = [IO.Path]::GetFileNameWithoutExtension($name)

        # Normalize line endings to LF (bash on MSYS chokes on CRLF scripts).
        $raw = [IO.File]::ReadAllBytes($s.FullName)
        if ($raw -contains [byte]13) {
            $fixed = [byte[]]($raw | Where-Object { $_ -ne 13 })
            [IO.File]::WriteAllBytes($s.FullName, $fixed)
            Write-Host "  normalized CRLF -> LF in $name"
        }

        $sw = [Diagnostics.Stopwatch]::StartNew()
        $r1 = Invoke-GitBash $name 1
        $r2 = Invoke-GitBash $name 2
        $sw.Stop()

        if ($r1.TimedOut -or $r2.TimedOut) {
            $problems += "$name : TIMEOUT"
            Write-Host ("TIMEOUT {0}" -f $name)
            continue
        }
        $same = ($r1.Code -eq $r2.Code) -and
                ([Convert]::ToBase64String($r1.Stdout) -eq [Convert]::ToBase64String($r2.Stdout))
        if (-not $same) {
            $problems += "$name : FLAKY (exit $($r1.Code) vs $($r2.Code), $($r1.Stdout.Length) vs $($r2.Stdout.Length) bytes)"
            Write-Host ("FLAKY   {0}" -f $name)
            continue
        }

        [IO.File]::WriteAllBytes((Join-Path $scriptsDir "$base.out"), $r1.Stdout)
        [IO.File]::WriteAllText((Join-Path $scriptsDir "$base.code"), [string]$r1.Code)
        $written++
        $lines = ([Text.Encoding]::UTF8.GetString($r1.Stdout) -split "`n").Count - 1
        Write-Host ("ok      {0,-36} exit={1,-3} lines={2,-4} {3,6:N1}s" -f $name, $r1.Code, $lines, $sw.Elapsed.TotalSeconds)
        if ($r1.Stderr.Length -gt 0) {
            Write-Host "        (stderr not empty; first lines:)"
            ($r1.Stderr -split "`n" | Select-Object -First 5) | ForEach-Object { Write-Host "        | $_" }
        }
    }
}
finally {
    Remove-Item -LiteralPath $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "wrote expected output for $written/$(@($scripts).Count) scripts"
if ($problems.Count -gt 0) {
    Write-Host "problems:"
    $problems | ForEach-Object { Write-Host "  $_" }
    exit 1
}
exit 0
