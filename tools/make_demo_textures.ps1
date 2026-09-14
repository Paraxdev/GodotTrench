# Generates the placeholder textures used by the Godot demo project.
param([string]$Root = "$PSScriptRoot\..\godot\demo\textures")
Add-Type -AssemblyName System.Drawing

function New-Texture([string]$Path, [int]$Size, [scriptblock]$Paint) {
    New-Item -ItemType Directory -Force (Split-Path $Path) | Out-Null
    $bmp = New-Object System.Drawing.Bitmap $Size, $Size
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    & $Paint $g $Size
    $g.Dispose()
    $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

function Brush([int]$r, [int]$g, [int]$b) { New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, $r, $g, $b)) }
function Pen([int]$r, [int]$g, [int]$b, [float]$w) { New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(255, $r, $g, $b)), $w }

New-Texture "$Root\base\floor.png" 128 {
    param($g, $s)
    $g.FillRectangle((Brush 96 92 86), 0, 0, $s, $s)
    for ($i = 0; $i -le $s; $i += 32) {
        $g.DrawLine((Pen 70 66 60 2), $i, 0, $i, $s)
        $g.DrawLine((Pen 70 66 60 2), 0, $i, $s, $i)
    }
}

New-Texture "$Root\base\wall.png" 128 {
    param($g, $s)
    $g.FillRectangle((Brush 150 80 60), 0, 0, $s, $s)
    for ($row = 0; $row -lt 8; $row++) {
        $y = $row * 16
        $g.DrawLine((Pen 200 190 180 2), 0, $y, $s, $y)
        $offset = if ($row % 2 -eq 0) { 0 } else { 16 }
        for ($x = $offset; $x -le $s; $x += 32) { $g.DrawLine((Pen 200 190 180 2), $x, $y, $x, $y + 16) }
    }
}

New-Texture "$Root\base\metal.png" 64 {
    param($g, $s)
    $g.FillRectangle((Brush 110 120 130), 0, 0, $s, $s)
    $g.DrawRectangle((Pen 60 70 80 4), 2, 2, $s - 4, $s - 4)
    foreach ($p in @(8, 52)) { foreach ($q in @(8, 52)) { $g.FillEllipse((Brush 170 180 190), $p, $q, 5, 5) } }
}

# Same stripe colors as the editor's built-in tool textures (gt_editor materials.rs).
$special = @{
    clip    = @(@(90, 20, 90), @(180, 40, 180))
    skip    = @(@(40, 40, 40), @(90, 90, 90))
    origin  = @(@(120, 70, 10), @(230, 140, 20))
    trigger = @(@(140, 90, 10), @(230, 160, 30))
}
foreach ($name in $special.Keys) {
    $dark, $light = $special[$name]
    New-Texture "$Root\special\$name.png" 64 {
        param($g, $s)
        $g.FillRectangle((Brush $dark[0] $dark[1] $dark[2]), 0, 0, $s, $s)
        for ($i = -$s; $i -lt $s * 2; $i += 16) { $g.DrawLine((Pen $light[0] $light[1] $light[2] 6), $i, 0, $i + $s, $s) }
    }
}
Get-ChildItem -Recurse $Root -Filter *.png | Select-Object FullName, Length
