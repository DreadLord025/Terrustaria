$mine = Get-Content crates\tr-app\src\npc.rs -Encoding UTF8
$g62 = Get-Content $env:TEMP\g62_npc.rs -Encoding UTF8

function ExtractFn($lines) {
    $s = -1; $e = -1
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match 'UpdateNPC_UpdateTrails.*90779-90875.*per-type') { $s = $i; break }
    }
    for ($j = $s; $j -lt $lines.Count; $j++) {
        if ($lines[$j] -match 'self\.old_rot\[0\] = self\.rotation;') { $e = $j + 1; break }
    }
    return , @($lines[$s..$e])
}
$a = ExtractFn $mine; $b = ExtractFn $g62
"update_trails: mine={0} g62={1} equal={2}" -f $a.Count, $b.Count, (($a -join "`n") -ceq ($b -join "`n"))
if (($a -join "`n") -cne ($b -join "`n")) {
    for ($k = 0; $k -lt [Math]::Max($a.Count, $b.Count); $k++) {
        if ($a[$k] -cne $b[$k]) { "DIFF at {0}:`nMINE: {1}`nG62 : {2}" -f $k, $a[$k], $b[$k]; break }
    }
}

function ExtractTransform($lines) {
    $s = -1
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match 'trail resize \+ zero loop') { $s = $i; break }
    }
    return , @($lines[$s..($s + 6)])
}
$a = ExtractTransform $mine; $b = ExtractTransform $g62
"transform: mine={0} g62={1} equal={2}" -f $a.Count, $b.Count, (($a -join "`n") -ceq ($b -join "`n"))

function ExtractWit($lines) {
    $s = -1; $e = -1
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($s -lt 0 -and $lines[$i] -match 'W4-g60 wiring .*UpdateNPC_UpdateTrails.*402 mode-7 arm') { $s = $i }
        if ($s -ge 0 -and $lines[$i] -match 'pub\(super\) fn bare') { $e = $i - 1; break }
    }
    return , @($lines[$s..$e])
}
$a = ExtractWit $mine; $b = ExtractWit $g62
"witnesses: mine={0} g62={1} equal={2}" -f $a.Count, $b.Count, (($a -join "`n") -ceq ($b -join "`n"))
if (($a -join "`n") -cne ($b -join "`n")) {
    for ($k = 0; $k -lt [Math]::Max($a.Count, $b.Count); $k++) {
        if ($a[$k] -cne $b[$k]) { "DIFF at {0}:`nMINE: {1}`nG62 : {2}" -f $k, $a[$k], $b[$k]; break }
    }
}

# data.rs table
$mineD = Get-Content crates\tr-app\src\npc\data.rs -Encoding UTF8
$g62D = Get-Content $env:TEMP\g62_data.rs -Encoding UTF8
function ExtractTable($lines) {
    $s = -1; $e = -1
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($s -lt 0 -and $lines[$i] -match 'NPCID\.Sets\.TrailCacheLength.*NPCID\.cs:10450.*FULL') { $s = $i }
        if ($s -ge 0 -and $lines[$i] -match 'pub fn trail_cache_length') {
            for ($j = $i; $j -lt $lines.Count; $j++) { if ($lines[$j] -match '^    \}$') { $e = $j; break } }
            break
        }
    }
    return , @($lines[$s..$e])
}
$a = ExtractTable $mineD; $b = ExtractTable $g62D
"table: mine={0} g62={1} equal={2}" -f $a.Count, $b.Count, (($a -join "`n") -ceq ($b -join "`n"))
if (($a -join "`n") -cne ($b -join "`n")) {
    for ($k = 0; $k -lt [Math]::Max($a.Count, $b.Count); $k++) {
        if ($a[$k] -cne $b[$k]) { "DIFF at {0}:`nMINE: {1}`nG62 : {2}" -f $k, $a[$k], $b[$k]; break }
    }
}
