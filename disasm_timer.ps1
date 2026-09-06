$assemblyPath = "C:\Program Files (x86)\HEOSD\Machenike Control Center.exe"
$bytes = [System.IO.File]::ReadAllBytes($assemblyPath)
$asm = [System.Reflection.Assembly]::Load($bytes)

function Disassemble-Method($type, $name) {
    $m = $type.GetMethod($name, [System.Reflection.BindingFlags]'Static,Instance,Public,NonPublic')
    if (-not $m) { Write-Host "Method $name not found"; return }
    Write-Host "================ $name ================"
    $body = $m.GetMethodBody()
    if (-not $body) { return }
    $il = $body.GetILAsByteArray()
    $module = $m.Module
    
    $opcodes = @{}
    foreach ($field in [System.Reflection.Emit.OpCodes].GetFields([System.Reflection.BindingFlags]'Public,Static')) {
        $op = $field.GetValue($null)
        $opcodes[$op.Value] = $op
    }

    $i = 0
    while ($i -lt $il.Length) {
        $offset = $i
        $b = $il[$i++]
        $op = if ($b -eq 0xfe) { $opcodes[[int16]((0xfe -shl 8) -bor $il[$i++])] } else { $opcodes[[int16]$b] }
        $operandStr = ""
        if ($op) {
            switch ($op.OperandType) {
                InlineNone {}
                ShortInlineI { $raw = $il[$i++]; $val = if ($raw -gt 127) { $raw - 256 } else { $raw }; $operandStr = "$val" }
                InlineI { $val = [System.BitConverter]::ToInt32($il, $i); $i += 4; $operandStr = "$val (0x$($val.ToString('X')))" }
                InlineMethod {
                    $token = [System.BitConverter]::ToInt32($il, $i); $i += 4
                    try { $resolved = $module.ResolveMember($token); $operandStr = "$($resolved.DeclaringType.Name)::$($resolved.Name)" } catch { $operandStr = "Token: 0x$($token.ToString('X'))" }
                }
                InlineField {
                    $token = [System.BitConverter]::ToInt32($il, $i); $i += 4
                    try { $resolved = $module.ResolveMember($token); $operandStr = "$($resolved.DeclaringType.Name)::$($resolved.Name)" } catch { $operandStr = "Token: 0x$($token.ToString('X'))" }
                }
                InlineString {
                    $token = [System.BitConverter]::ToInt32($il, $i); $i += 4
                    try { $str = $module.ResolveString($token); $operandStr = "`"$str`"" } catch { $operandStr = "Token: 0x$($token.ToString('X'))" }
                }
                ShortInlineBrTarget {
                    $raw = $il[$i++]; $val = if ($raw -gt 127) { $raw - 256 } else { $raw }; $operandStr = "IL_{0:X4}" -f ($i + $val)
                }
                InlineBrTarget {
                    $val = [System.BitConverter]::ToInt32($il, $i); $i += 4; $operandStr = "IL_{0:X4}" -f ($i + $val)
                }
                default { }
            }
        }
        Write-Host ("  IL_{0:X4}: {1,-12} {2}" -f $offset, $op.Name, $operandStr)
    }
}

$modelPageType = $asm.GetType("Machenike_Control_Center.ModelSwitchPage")
Disassemble-Method $modelPageType "timer_Tick"
Disassemble-Method $modelPageType "FanSpeed_RotationAnimation"
