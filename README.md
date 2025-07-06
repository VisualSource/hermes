# Hermes

A simple discord like clone using RTCPeerConnection and websockets to handle communications


## setup

1. cd project directory
2. `bun install`

3. Create a `.env` file in the root project dir.
4. Add the following env keys

`VITE_SERVER_HOST="localhost:5000"`
Sets the server host for the client

`VITE_USE_INVALID_CERT=1`
If using a self signed tls cert set this to 1 else 0

`VITE_DEFAULT_CHANNEL_ID="001417d2-c76c-4622-9e75-bb6303341cd0"`
the default channel the client loads
> the current impl does not support switching channels

5. cd server 
6. `bun install`

7. Add a `.env` to the server directory, and the following keys

`TLS_CERT="../certs/ca-cert.pem"`
`TLS_KEY="../certs/ca-key.pem"`
`TLS_PASSKEY=""`

`JWT_SECRET=""`
`JWT_REFRESH_SECRET=""`

`DEFAULT_CHANNEL_ID="001417d2-c76c-4622-9e75-bb6303341cd0"`
This should be the same as `VITE_DEFAULT_CHANNEL_ID`

### self signed cert setup

https://www.thewindowsclub.com/create-self-signed-ssl-certificates-in-windows-10
```powershell
New-SelfSignedCertificate -CertStoreLocation Cert:\LocalMachine\My -DnsName "localhost:5000" -FriendlyName "LoadingZoneHermes" -NotAfter (Get-Date).AddYears(10)
```

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
