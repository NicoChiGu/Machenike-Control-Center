; NSIS Custom Hooks for Machenike L16W Center
; Handles automated installation, configuration and startup of the WTIOportDrv kernel driver service

!macro NSIS_HOOK_PREINSTALL
  ; Stop service if it was running before upgrade/reinstall
  nsExec::Exec 'sc.exe stop ioportdrv'
!macroend

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Installing WTIOportDrv kernel driver..."

  ; 1. Copy the signed driver to Windows System32\drivers directory
  CopyFiles /SILENT "$INSTDIR\resources\WTIOportDrv.sys" "$SYSDIR\drivers\WTIOportDrv.sys"

  ; 2. Create the kernel driver service if it does not already exist
  nsExec::Exec 'sc.exe create ioportdrv binPath= "$SYSDIR\drivers\WTIOportDrv.sys" type= kernel start= auto'

  ; 3. Ensure the service binPath always points to the System32\drivers location
  nsExec::Exec 'sc.exe config ioportdrv binPath= "$SYSDIR\drivers\WTIOportDrv.sys" type= kernel start= auto'

  ; 4. Start the service
  nsExec::Exec 'sc.exe start ioportdrv'

  DetailPrint "WTIOportDrv driver installed and started successfully."
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Stopping and removing ioportdrv service..."

  ; Stop the kernel driver service
  nsExec::Exec 'sc.exe stop ioportdrv'

  ; Delete the kernel driver service entry
  nsExec::Exec 'sc.exe delete ioportdrv'

  ; Clean up the driver binary from system drivers
  Delete "$SYSDIR\drivers\WTIOportDrv.sys"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Post-uninstall cleanup if needed
!macroend
