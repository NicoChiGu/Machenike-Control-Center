$assemblyPath = "C:\Program Files (x86)\HEOSD\Machenike Control Center.exe"
$bytes = [System.IO.File]::ReadAllBytes($assemblyPath)
$asm = [System.Reflection.Assembly]::Load($bytes)

foreach ($type in $asm.GetTypes()) {
    foreach ($m in $type.GetMethods([System.Reflection.BindingFlags]'Static,Instance,Public,NonPublic')) {
        $body = $m.GetMethodBody()
        if (-not $body) { continue }
        $il = $body.GetILAsByteArray()
        $ilStr = [System.BitConverter]::ToString($il)
        
        # Check if method calls ECManager methods
        # Let's inspect tokens in method
        $module = $m.Module
        $body = $m.GetMethodBody()
        $il = $body.GetILAsByteArray()
        
        for ($i = 0; $i -lt $il.Length - 4; $i++) {
            if ($il[$i] -in @(0x28, 0x6F)) { # call or callvirt
                $token = [System.BitConverter]::ToInt32($il, $i + 1)
                try {
                    $member = $module.ResolveMember($token)
                    if ($member.DeclaringType.Name -eq "ECManager") {
                        Write-Host "$($type.Name)::$($m.Name) calls $($member.Name)"
                    }
                } catch {}
            }
        }
    }
}
