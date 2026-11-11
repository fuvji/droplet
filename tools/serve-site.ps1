param(
    [int]$Port = 4173
)

$ErrorActionPreference = 'Stop'
$root = Join-Path (Split-Path -Parent $PSScriptRoot) 'site\public'
$rootFull = [IO.Path]::GetFullPath($root).TrimEnd('\')
$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, $Port)

$mime = @{
    '.html' = 'text/html; charset=utf-8'
    '.css'  = 'text/css; charset=utf-8'
    '.js'   = 'text/javascript; charset=utf-8'
    '.png'  = 'image/png'
    '.svg'  = 'image/svg+xml'
    '.webm' = 'video/webm'
    '.xml'  = 'application/xml; charset=utf-8'
    '.txt'  = 'text/plain; charset=utf-8'
    '.exe' = 'application/octet-stream'
}

$listener.Start()
Write-Output "Droplet site: http://127.0.0.1:$Port/"

try {
    while ($true) {
        $client = $listener.AcceptTcpClient()
        try {
            $stream = $client.GetStream()
            $reader = New-Object IO.StreamReader($stream, [Text.Encoding]::ASCII, $false, 1024, $true)
            $requestLine = $reader.ReadLine()
            while (($line = $reader.ReadLine()) -ne $null -and $line -ne '') { }

            $status = '200 OK'
            $body = [byte[]]@()
            $type = 'text/plain; charset=utf-8'

            if ($requestLine -match '^(GET|HEAD) ([^ ]+) HTTP/1\.[01]$') {
                $method = $Matches[1]
                $path = [Uri]::UnescapeDataString(($Matches[2] -split '\?', 2)[0])
                if ($path -eq '/') { $path = '/index.html' }
                $candidate = [IO.Path]::GetFullPath((Join-Path $rootFull ($path.TrimStart('/').Replace('/', '\'))))
                if (-not (Test-Path -LiteralPath $candidate -PathType Leaf) -and
                    [string]::IsNullOrEmpty([IO.Path]::GetExtension($candidate))) {
                    $htmlCandidate = $candidate + '.html'
                    if (Test-Path -LiteralPath $htmlCandidate -PathType Leaf) {
                        $candidate = $htmlCandidate
                    }
                }
                if ($candidate.StartsWith($rootFull, [StringComparison]::OrdinalIgnoreCase) -and
                    (Test-Path -LiteralPath $candidate -PathType Leaf)) {
                    $body = [IO.File]::ReadAllBytes($candidate)
                    $extension = [IO.Path]::GetExtension($candidate).ToLowerInvariant()
                    if ($mime.ContainsKey($extension)) { $type = $mime[$extension] }
                    if ($method -eq 'HEAD') { $body = [byte[]]@() }
                } else {
                    $status = '404 Not Found'
                    $body = [Text.Encoding]::UTF8.GetBytes('Not found')
                }
            } else {
                $status = '400 Bad Request'
                $body = [Text.Encoding]::UTF8.GetBytes('Bad request')
            }

            $header = "HTTP/1.1 $status`r`nContent-Type: $type`r`nContent-Length: $($body.Length)`r`nCache-Control: no-store`r`nConnection: close`r`nX-Content-Type-Options: nosniff`r`n`r`n"
            $headerBytes = [Text.Encoding]::ASCII.GetBytes($header)
            $stream.Write($headerBytes, 0, $headerBytes.Length)
            if ($body.Length -gt 0) { $stream.Write($body, 0, $body.Length) }
            $stream.Flush()
        } finally {
            $client.Close()
        }
    }
} finally {
    $listener.Stop()
}
