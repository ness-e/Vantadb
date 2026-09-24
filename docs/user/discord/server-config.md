---
title: "Server Configuration"
type: discord
status: active
tags: [vantadb, discord]
last_reviewed: 2026-07-21
---

# Server Configuration

## Categories & Channels

### 👋 WELCOME (all members)
| Channel | Type | Topic |
|---|---|---|
| #📜rules | Text | Server rules (read-only) |
| #🎭roles | Text | Self-assign language and technology roles (reactions) |
| #📢announcements | Announcement | Official releases and GitHub activity (read-only) |

### 💬 COMMUNITY (all members)
| Channel | Type | Topic |
|---|---|---|
| #🌐showcase | Forum | Show off projects built with VantaDB |
| #🌐general | Text | General discussion (English & Spanish, use language roles) |
| #🗣️off-topic | Text | Casual chat |

### 🛠️ DEV (all members)
| Channel | Type | Topic |
|---|---|---|
| #❓help | Forum | Questions and support |
| #🐛bug-reports | Forum | Bug reports |
| #💻dev-chat | Text | Technical development discussion |
| #💡ideas | Forum | Feature suggestions |

### 🎤 EVENTS (all members)
| Channel | Type | Topic |
|---|---|---|
| #🎤 Stage | Voice | AMAs, presentations, community events |

### 🛡️ STAFF (Admin only)
| Channel | Type |
|---|---|
| staff-chat | Text |
| mod-log | Text |

## Roles

### Staff Roles
| Role | Color | Permissions |
|---|---|---|
| VantaDB | `#e74c3c` | Administrator (managed bot) |
| Admin | `#ff5500` | Administrator |
| Maintainer | `#ea580c` | Kick, Manage Messages, Manage Roles |
| Contributor | `#8b5cf6` | Send Messages, Embed, Attach |

### Progression Roles
| Role | Color | Permissions |
|---|---|---|
| Member | `#6b7280` | ViewChannel, SendMessages, Embed, Attach |
| New Member | `#374151` | ViewChannel only |

### Identity Roles (self-assign via reaction roles)
| Role | Color | Notes |
|---|---|---|
| English Speaker | `#3b82f6` | Language identity |
| Spanish Speaker | `#eab308` | Language identity |
| Rustacean | `#e0e0e0` | Cosmetic |
| Pythonista | `#e0e0e0` | Cosmetic |
| TypeScript | `#e0e0e0` | Cosmetic |
| AI/ML | `#e0e0e0` | Cosmetic |

### System Roles
| Role | Purpose |
|---|---|
| carl-bot | Bot automation |
| @everyone | Base permissions |

## Permission Architecture

All categories grant ViewChannel + SendMessages to @everyone. **Single-channel bilingual model** — English and Spanish coexist in the same channels, language roles are cosmetic/identity only. Only STAFF category is restricted to Admin role.

## Server Settings

| Setting | Value |
|---|---|
| Verification Level | 1 (LOW — verified email) |
| Default Notifications | @mentions only |
| Explicit Media Filter | Scan all media |
| Server Widget | Enabled |

## Bot Stack

| Bot | Purpose | Status |
|---|---|---|
| VantaDB (custom) | Server management via API | Active |
| Carl-bot | Reaction roles, autorole, moderation | Installed — needs dashboard config |

## Integrations

## Integrations

### GitHub → Discord webhook

A **repository webhook** (GitHub events → Discord channel webhook URL) forwards
GitHub activity to Discord. It is a standard GitHub repo webhook whose **Payload
URL** is a Discord channel webhook URL; GitHub delivers native event payloads to
Discord, which renders them as messages in `#📢announcements`.

| Event type | GitHub trigger (Settings → Webhooks → events) | Destination |
|---|---|---|
| `push` | Pushes to any branch (default: all branches) | #📢announcements |
| `pull_request` | PR opened, closed, merged, reopened | #📢announcements |
| `issues` | Issue opened, closed, reopened, labeled | #📢announcements |
| `release` | Release published | #📢announcements |

#### Adding a new event type

Events are selected in the GitHub webhook configuration, not in Discord:

1. Go to the repository → **Settings → Webhooks**.
2. Click **Edit** on the active GitHub→Discord webhook (or **Add webhook** to create one).
3. Under **Let me select individual events**, check the desired event (e.g.
   `Pushes`, `Pull requests`, `Issues`, `Releases`, `Fork`, `Star`).
4. **Update webhook** to save. No Discord-side change is required — the payload
   is forwarded to the same channel webhook URL.

#### Where it is configured

- **GitHub:** repository → **Settings → Webhooks**. The webhook's **Payload URL**
  is the Discord channel webhook endpoint
  (`https://discord.com/api/webhooks/<id>/<token>`).
- **Discord:** channel → **Edit Channel → Integrations → Webhooks**. The webhook
  URL used by the GitHub webhook must exist here and have **Send Messages**
  permission for the target channel. Channel per event is fixed by which Discord
  webhook URL the GitHub webhook points at; a separate event → channel mapping
  would require a second webhook, not Discord-side routing.
