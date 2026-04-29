# SSH Tunnel Configuration — workspace-tunnel

## Active Tunnel

| Field | Value |
|---|---|
| tmux window | `workspace-tunnel` |
| remote host | `buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com` |
| local ports forwarded | `5432` (PostgreSQL), `8088` (SeatLoom frontend) |
| SSH options | `-o ServerAliveInterval=60 -o ServerAliveCountMax=3 -CAXY` |
| keep-alive | 60s interval, 3 max failures |

## Usage

### PostgreSQL (DBeaver)
```
host: localhost
port: 5432
user: seatloom
password: seatloom
database: seatloom
```

### Frontend (Chrome)
```
http://localhost:8088
```

## Remote Environment
- OS: Ubuntu 22.04.5 LTS (x86_64)
- Docker: PostgreSQL 16 container on `0.0.0.0:5432`
- Workspace: `/data/seatloom-verify/repo` (SeatLoom repo)
- Rust: 1.95.0 (cargo at `$HOME/.cargo/bin`)

## Notes
- Tunnel must be alive before connecting DBeaver or browser
- If tunnel drops, restart in tmux window `workspace-tunnel`
- Port 8088 serves the Tauri dev frontend from the remote workspace
- Do NOT use the `qy.machdrive.cn` tunnel (that's QL resource); this one uses `platform.shaipower.com`
