# Installing the build requirements:
# 1) https://aka.ms/dotnet-download
# 2) Open powershell
# 3) dotnet nuget add source https://api.nuget.org/v3/index.json -n nuget.org
# 4) dotnet tool install - -global wix
# 5) & $Env:USERPROFILE\.dotnet\tools\wix extension add -g WixToolset.UI.wixext
#
# You can run this script by opening a powershell session with a relaxed execution policy:
#   start -> run -> powershell -ExecutionPolicy RemoteSigned
#   & msi\build.ps1 "1.2.3"
# Replace 1.2.3 with the proper version number of the application

# If the MSI installer encounters an error you can debug by following these steps:
# https://learn.microsoft.com/en-us/troubleshoot/windows-client/application-management/enable-windows-installer-logging

Push-Location
Set-Location -Path "$PSScriptRoot"
$version = $args[0]

try
{
    & $Env:USERPROFILE\.dotnet\tools\wix build package.wxs -arch x64 -ext WixToolset.UI.wixext -d "Version=$version" -out ..\target\msi\groovtube-hotkey.msi
}
finally
{
    Pop-Location
}
