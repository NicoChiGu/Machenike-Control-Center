@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

echo ========================================================
echo   Machenike L16W Center - 驱动独立安装与服务配置工具
echo ========================================================
echo.

:: 检查是否具有管理员权限
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [提示] 正在请求管理员权限以配置底层硬件驱动服务...
    powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process cmd -ArgumentList '/c \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

cd /d "%~dp0"

echo [1/3] 正在复制官方 WHQL 签名驱动至 Windows 系统驱动目录...
set DRV_SRC=""
if exist "driver\WTIOportDrv.sys" (
    set DRV_SRC="driver\WTIOportDrv.sys"
) else if exist "src-tauri\resources\WTIOportDrv.sys" (
    set DRV_SRC="src-tauri\resources\WTIOportDrv.sys"
) else (
    echo [错误] 未在项目目录内找到 WTIOportDrv.sys 驱动文件！
    pause
    exit /b 1
)

copy /y !DRV_SRC! "%SystemRoot%\System32\drivers\WTIOportDrv.sys" >nul
if %errorLevel% neq 0 (
    echo [错误] 复制驱动文件至 %SystemRoot%\System32\drivers 失败！
    pause
    exit /b 1
)
echo       -> 驱动已成功放置于: %SystemRoot%\System32\drivers\WTIOportDrv.sys

echo.
echo [2/3] 正在配置 ioportdrv 内核驱动服务...
sc.exe stop ioportdrv >nul 2>&1
sc.exe create ioportdrv binPath= "%SystemRoot%\System32\drivers\WTIOportDrv.sys" type= kernel start= auto >nul 2>&1
sc.exe config ioportdrv binPath= "%SystemRoot%\System32\drivers\WTIOportDrv.sys" type= kernel start= auto >nul 2>&1

echo.
echo [3/3] 正在启动 ioportdrv 服务...
sc.exe start ioportdrv >nul 2>&1

echo.
echo ========================================================
echo [成功] 驱动服务已成功安装并独立化！
echo.
sc.exe query ioportdrv | findstr "STATE"
echo.
echo 现在，本控制中心已彻底独立于原厂软件。
echo 您可以放心地卸载或物理删除 C:\Program Files (x86)\HEOSD 文件夹。
echo ========================================================
echo.
pause
