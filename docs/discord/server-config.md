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

All categories grant ViewChannel + SendMessages to @everyone (no EN/ES separation — single-channel bilingual model). Only STAFF category is restricted to Admin role.

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

| Integration | Events | Channel |
|---|---|---|
| GitHub Webhook | push, pull_request, issues, release | #📢announcements |
