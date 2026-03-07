<div align="center">

# QQ Farm Bot Panel

![Tauri](https://img.shields.io/badge/Tauri_2-FFC131?style=flat-square&logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React_19-61DAFB?style=flat-square&logo=react&logoColor=black)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white)
![Vite](https://img.shields.io/badge/Vite_7-646CFF?style=flat-square&logo=vite&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_4-06B6D4?style=flat-square&logo=tailwindcss&logoColor=white)

**QQ 农场自动化管理面板**

一个基于 Tauri 2 构建的跨平台桌面客户端，通过 WebSocket 直连 QQ 农场游戏服务器，提供完整的农场自动化管理能力。

![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey?style=flat-square)
![Version](https://img.shields.io/badge/version-0.0.1-blue?style=flat-square)

> **WIP** — 自动化引擎尚未实现，当前版本仅支持面板查看和手动操作。下方标注的自动化功能为规划中的目标功能。

</div>

---

## Features

### 连接与登录

- **Code 登录认证** — 通过抓包获取登录 Code，支持手动输入或本地端口 7788 自动接收
- **WebSocket 长连接** — 直连游戏服务器，Protobuf 协议编解码
- **连接状态管理** — 实时显示连接状态，断开后自动清理会话数据

### 农场自动化 `planned`

| 功能 | 说明 |
|------|------|
| 自动收获播种 | 成熟作物自动收获，空地自动播种 |
| 智能种植策略 | 支持指定种子 / 最高等级 / 最大经验 / 最大利润四种策略 |
| 农场打理 | 自动浇水、除草、除虫，可分别开关 |
| 施肥管理 | 普通 / 有机化肥策略，自动填充化肥槽，不足时自动购买 |
| 土地升级 | 金币充足时自动升级土地等级 |
| 推送触发巡田 | 收到服务器推送时立即检查农场 |

### 社交系统 `planned`

| 功能 | 说明 |
|------|------|
| 好友农场巡逻 | 自动巡查好友农场，一键批量操作 |
| 自动偷菜 | 偷取好友成熟作物 |
| 自动帮忙 | 帮好友浇水、除草、除虫，经验上限可停 |
| 好友静默时段 | 指定时间段内暂停好友巡查 |

### 任务与奖励 `planned`

- 自动完成日常任务并领取奖励
- 自动领取邮件附件
- 自动领取商城免费礼包、VIP 礼包、月卡奖励、开服红包
- 自动分享奖励

### 管理面板

- **仪表盘** — 总览农场状态、连接信息、收益估算
- **农场** — 查看土地详情、作物进度、手动操作
- **好友** — 好友列表筛选（可偷 / 旱 / 草 / 虫 / 无操作）、单独或批量巡逻
- **仓库** — 背包物品浏览，种子 / 果实 / 道具分类查看
- **任务** — 任务列表分类展示，活跃度和奖励进度
- **设置** — 全部自动化配置项，可折叠分组
- **日志** — 实时操作日志流

---

## Tech Stack

```
Frontend                Backend                Protocol
├─ React 19             ├─ Rust                ├─ Protobuf (prost)
├─ TypeScript 5.8       ├─ Tauri 2.x           ├─ WebSocket (tungstenite)
├─ Vite 7               ├─ Tokio async runtime └─ Proto3 (16 定义文件)
├─ Tailwind CSS 4       ├─ parking_lot RwLock
├─ React Router 7       └─ reqwest HTTP
└─ Lucide Icons
```

---

## Getting Started

### 环境要求

| 依赖 | 版本要求 |
|------|---------|
| **Node.js** | >= 20 |
| **pnpm** | >= 9 |
| **Rust** | >= 1.80 (stable) |
| **Tauri CLI** | 2.x (`pnpm tauri --version`) |

macOS 额外需要 Xcode Command Line Tools；Windows 需要 [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)；Linux 需要 `libwebkit2gtk-4.1-dev` 等系统依赖，详见 [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)。

### 安装

```bash
# 克隆项目
git clone https://github.com/your-username/qq-farm-bot-panel.git
cd qq-farm-bot-panel

# 安装前端依赖
pnpm install
```

### 开发

```bash
# 启动开发模式（前端热更新 + Rust 后端）
pnpm tauri dev
```

前端 dev server 运行在 `http://localhost:1420`，Tauri 窗口自动打开。

### 构建

```bash
# 构建生产版本
pnpm tauri build
```

产物位于 `src-tauri/target/release/bundle/`：

| 平台 | 产物 |
|------|------|
| macOS | `.dmg` / `.app` |
| Windows | `.msi` / `.exe` (NSIS) |
| Linux | `.deb` / `.AppImage` |

### 移动端（实验性）

```bash
# 初始化移动端支持
pnpm tauri android init
pnpm tauri ios init

# 开发
pnpm tauri android dev
pnpm tauri ios dev
```

---

## Project Structure

```
qq-farm-bot-panel/
├── src/                        # React 前端
│   ├── api/                    # Tauri IPC 调用封装
│   ├── components/             # 通用组件 (Button, Card, Toast, PageHeader...)
│   ├── hooks/                  # 自定义 Hooks (useStatus, useTauriEvent...)
│   ├── pages/                  # 页面 (Dashboard, Farm, Friends, Inventory...)
│   ├── data/                   # 静态数据 (种子信息等)
│   └── types/                  # TypeScript 类型定义
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── commands/           # Tauri 命令 (前端可调用的 API)
│   │   ├── network/            # WebSocket 连接管理
│   │   ├── proto/              # Protobuf 定义与生成代码
│   │   ├── game/               # 游戏逻辑 (农场、好友、任务...)
│   │   └── automation/         # 自动化引擎
│   ├── proto/                  # .proto 源文件
│   └── Cargo.toml
├── package.json
└── CLAUDE.md
```

---

## License

MIT
