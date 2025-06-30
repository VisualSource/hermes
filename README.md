# Tauri + React + Typescript


## Setup self signed cert 

https://www.thewindowsclub.com/create-self-signed-ssl-certificates-in-windows-10
```powershell
New-SelfSignedCertificate -CertStoreLocation Cert:\LocalMachine\My -DnsName "localhost:5000" -FriendlyName "LoadingZoneHermes" -NotAfter (Get-Date).AddYears(10)
```
This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
