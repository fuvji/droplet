param(
    [string]$AssetDir = "packaging/Assets"
)

# One canonical mark for every surface: a solid ivory droplet with no plate,
# border, glow, or baked background. The same path is used by the SVG favicon
# and the native tray/window icon.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root $AssetDir
New-Item -ItemType Directory -Force -Path $out | Out-Null

Add-Type -AssemblyName System.Drawing

function New-DropletPoint([single]$px, [single]$py) {
    return New-Object System.Drawing.PointF($px, $py)
}

function New-DropletPath([single]$x, [single]$y, [single]$w, [single]$h) {
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.StartFigure()
    $path.AddBezier(
        (New-DropletPoint ($x + 0.5000 * $w) ($y + 0.0250 * $h)),
        (New-DropletPoint ($x + 0.4250 * $w) ($y + 0.2080 * $h)),
        (New-DropletPoint ($x + 0.2375 * $w) ($y + 0.4080 * $h)),
        (New-DropletPoint ($x + 0.2375 * $w) ($y + 0.5830 * $h))
    )
    $path.AddBezier(
        (New-DropletPoint ($x + 0.2375 * $w) ($y + 0.5830 * $h)),
        (New-DropletPoint ($x + 0.2375 * $w) ($y + 0.7290 * $h)),
        (New-DropletPoint ($x + 0.3540 * $w) ($y + 0.8450 * $h)),
        (New-DropletPoint ($x + 0.5000 * $w) ($y + 0.8450 * $h))
    )
    $path.AddBezier(
        (New-DropletPoint ($x + 0.5000 * $w) ($y + 0.8450 * $h)),
        (New-DropletPoint ($x + 0.6460 * $w) ($y + 0.8450 * $h)),
        (New-DropletPoint ($x + 0.7625 * $w) ($y + 0.7290 * $h)),
        (New-DropletPoint ($x + 0.7625 * $w) ($y + 0.5830 * $h))
    )
    $path.AddBezier(
        (New-DropletPoint ($x + 0.7625 * $w) ($y + 0.5830 * $h)),
        (New-DropletPoint ($x + 0.7625 * $w) ($y + 0.4080 * $h)),
        (New-DropletPoint ($x + 0.5750 * $w) ($y + 0.2080 * $h)),
        (New-DropletPoint ($x + 0.5000 * $w) ($y + 0.0250 * $h))
    )
    $path.CloseFigure()
    return $path
}

$sizes = @(
    [pscustomobject]@{ Name = 'StoreLogo.png';          Width = 50;  Height = 50 },
    [pscustomobject]@{ Name = 'Square44x44Logo.png';    Width = 44;  Height = 44 },
    [pscustomobject]@{ Name = 'Square71x71Logo.png';    Width = 71;  Height = 71 },
    [pscustomobject]@{ Name = 'Square150x150Logo.png';  Width = 150; Height = 150 },
    [pscustomobject]@{ Name = 'Square310x310Logo.png';  Width = 310; Height = 310 },
    [pscustomobject]@{ Name = 'Wide310x150Logo.png';    Width = 310; Height = 150 }
)

$ivory = [System.Drawing.ColorTranslator]::FromHtml('#F4F3EF')
foreach ($size in $sizes) {
    $bitmap = New-Object System.Drawing.Bitmap($size.Width, $size.Height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.Clear([System.Drawing.Color]::Transparent)
        $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
        $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
        $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

        # A square tile gets a centered mark; the wide tile stays visually
        # centered without stretching the droplet.
        $mark = [Math]::Min($size.Width, $size.Height) * 0.78
        $x = ($size.Width - $mark) * 0.5
        $y = ($size.Height - $mark * 1.03) * 0.5
        $path = New-DropletPath $x $y $mark ($mark * 1.08)
        $brush = New-Object System.Drawing.SolidBrush($ivory)
        try {
            $graphics.FillPath($brush, $path)
        } finally {
            $brush.Dispose()
            $path.Dispose()
        }

        $file = Join-Path $out $size.Name
        $bitmap.Save($file, [System.Drawing.Imaging.ImageFormat]::Png)
        Write-Output ("$($size.Name) $($size.Width)x$($size.Height) transparent")
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}
