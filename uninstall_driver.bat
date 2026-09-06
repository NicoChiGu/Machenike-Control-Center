@echo off
chcp 65001 >nul
setlocal

echo ========================================================
echo   Machenike L16W Center - 驱动服务清理与卸载工具
echo ========================================================
echo.

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [提示] 正在请求管理员权限...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process cmd -ArgumentList '/c \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

echo 正在停止 ioportdrv 服务...
sc.exe stop ioportdrv >nul 2>&1

echo 正在删除 ioportdrv 服务注册...
sc.exe delete ioportdrv >nul 2>&1

if exist "%SystemRoot%\System32\drivers\WTIOportDrv.sys" (
    echo 正在清理驱动文件 %SystemRoot%\System32\drivers\WTIOportDrv.sys ...
    del /f /q "%SystemRoot%\System32\drivers\WTIOportDrv.sys" >nul 2>&1
)

echo.
echo ========================================================
echo [成功] ioportdrv 驱动服务已停止并从系统中清理。
echo ========================================================
echo.
pause
