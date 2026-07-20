[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$OutputPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Outlook object-model constant. The numeric value is pinned here so the
# script does not require the Outlook primary interop assembly.
$olStoreANSI = 3

function Release-ComObject {
    param([object]$Value)
    if ($null -ne $Value -and [System.Runtime.InteropServices.Marshal]::IsComObject($Value)) {
        [void][System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($Value)
    }
}

$resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
$outputDirectory = Split-Path -Parent $resolvedOutput
if (-not (Test-Path -LiteralPath $outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory | Out-Null
}
if (Test-Path -LiteralPath $resolvedOutput) {
    throw "Refusing to overwrite existing file: $resolvedOutput"
}

$outlook = $null
$namespace = $null
$store = $null
$root = $null
$mailFolder = $null
$nestedFolder = $null
$nonMailFolder = $null
$tempAttachment = $null

try {
    $outlook = New-Object -ComObject Outlook.Application
    $namespace = $outlook.GetNamespace('MAPI')

    # AddStoreEx creates an Outlook 97-2002 ANSI PST when passed olStoreANSI.
    $namespace.AddStoreEx($resolvedOutput, $olStoreANSI)

    foreach ($candidate in @($namespace.Stores)) {
        if ([string]::Equals(
                [System.IO.Path]::GetFullPath([string]$candidate.FilePath),
                $resolvedOutput,
                [System.StringComparison]::OrdinalIgnoreCase)) {
            $store = $candidate
            break
        }
        Release-ComObject $candidate
    }
    if ($null -eq $store) {
        throw 'Outlook created the store but it could not be resolved by file path.'
    }

    $root = $store.GetRootFolder()
    $mailFolder = $root.Folders.Add('Synthetic Mail')
    $nestedFolder = $mailFolder.Folders.Add('Nested')
    $nonMailFolder = $root.Folders.Add('Typed Non-Mail')

    # Create every item directly in the generated PST. Outlook.Application.CreateItem
    # would create it in the profile's default store before a later move, which would
    # weaken the isolation and provenance guarantees for this controlled fixture.
    $plain = $mailFolder.Items.Add('IPM.Note')
    try {
        $plain.Subject = 'PSTD ANSI plain text baseline'
        $plain.To = 'to-one@example.test; to-two@example.test'
        $plain.CC = 'cc-one@example.test'
        $plain.BCC = 'bcc-one@example.test'
        $plain.Body = "Synthetic ANSI plain-text body.`r`nLine two.`r`n"
        $plain.Save()
    }
    finally {
        Release-ComObject $plain
    }

    $tempAttachment = Join-Path $env:TEMP 'pstd-ansi-fixture-attachment.txt'
    [System.IO.File]::WriteAllText(
        $tempAttachment,
        "PSTD ANSI fixture attachment`r`n",
        [System.Text.Encoding]::ASCII)

    $html = $nestedFolder.Items.Add('IPM.Note')
    try {
        $html.Subject = 'PSTD ANSI HTML and attachment baseline'
        $html.To = 'html-recipient@example.test'
        $html.Body = "Synthetic plain fallback for the HTML message.`r`n"
        $html.HTMLBody = '<html><body><p>Synthetic <strong>ANSI HTML</strong> body.</p></body></html>'
        [void]$html.Attachments.Add($tempAttachment)
        $html.Save()
    }
    finally {
        Release-ComObject $html
    }

    # Deliberately include one typed non-mail object. PSTD must classify it and
    # must never force it into EML output.
    $contact = $nonMailFolder.Items.Add('IPM.Contact')
    try {
        $contact.FullName = 'Synthetic Contact'
        $contact.Email1Address = 'contact@example.test'
        $contact.CompanyName = 'PSTD Fixture Data'
        $contact.Save()
    }
    finally {
        Release-ComObject $contact
    }

    # Detach the PST cleanly so Outlook flushes file metadata before hashing.
    $namespace.RemoveStore($root)
    Release-ComObject $root
    $root = $null
    Release-ComObject $store
    $store = $null

    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
    Start-Sleep -Seconds 2

    $bytes = [System.IO.File]::ReadAllBytes($resolvedOutput)
    if ($bytes.Length -lt 24) {
        throw "Generated PST is unexpectedly short: $($bytes.Length) bytes"
    }
    $magic = [System.Text.Encoding]::ASCII.GetString($bytes, 0, 4)
    $client = [System.Text.Encoding]::ASCII.GetString($bytes, 8, 2)
    $version = [System.BitConverter]::ToUInt16($bytes, 10)
    if ($magic -ne '!BDN' -or $client -ne 'SM' -or $version -notin @(14, 15)) {
        throw "Generated file is not an ANSI PST: magic=$magic client=$client version=$version"
    }

    $hash = (Get-FileHash -LiteralPath $resolvedOutput -Algorithm SHA256).Hash.ToLowerInvariant()
    [pscustomobject]@{
        path = $resolvedOutput
        bytes = $bytes.Length
        sha256 = $hash
        magic = $magic
        client = $client
        ndb_version = $version
    } | ConvertTo-Json
}
finally {
    if ($null -ne $tempAttachment -and (Test-Path -LiteralPath $tempAttachment)) {
        Remove-Item -LiteralPath $tempAttachment -Force
    }
    Release-ComObject $nonMailFolder
    Release-ComObject $nestedFolder
    Release-ComObject $mailFolder
    Release-ComObject $root
    Release-ComObject $store
    Release-ComObject $namespace
    Release-ComObject $outlook
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}
