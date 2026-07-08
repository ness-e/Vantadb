# Server Configuration

## Categories & Channels

### 1. 📋 INFO (shared — all members)
| Channel | Type | Topic |
|---|---|---|
| #📜rules | Text | Server rules (read-only, embed pinned) |
| #announcements | Text (Announcement) | Official releases and news (read-only) |
| #❓faq | Forum | Community FAQ — tags: setup, troubleshooting, installation, configuration, sdk, general |

### 2. 👋 WELCOME (shared — all members)
| Channel | Type | Topic |
|---|---|---|
| #👋introductions | Text | Introduce yourself (template pinned) |
| #🎭roles | Text | Self-assign language and notification roles |
| #ℹ️server-info | Text | Welcome info with quick links (embed pinned) |

### 3. 💬 COMMUNITY (Member only — language neutral)
| Channel | Type | Topic |
|---|---|---|
| #general-chat | Text | General discussion |
| #🌐showcase | Forum | Show off projects — tags: showcase, project, demo, tutorial, open-source |
| #💡ideas | Forum | Feature suggestions — tags: feature-request, enhancement, integration, ux, performance |
| #integrations | Text | Integrations, bots, tools that work with VantaDB |
| #📚resources | Text | Curated resources and links (embed pinned) |

### 4. 🛠️ DEV (Member only — language neutral)
| Channel | Type | Topic |
|---|---|---|
| #dev-chat | Text | Technical development discussion |
| #📋pr-reviews | Text | Pull request submissions and review |
| #🐛bug-reports | Forum | Bug reports — tags: confirmed, investigating, fixed, wontfix, critical, duplicate |
| #📊roadmap | Text | Development roadmap (embed pinned) |

### 5. 🇬🇧 ENGLISH (English Speaker role only)
| Channel | Type | Topic |
|---|---|---|
| #general | Text | General discussion in English |
| #dev-chat | Text | Technical discussion in English |
| #off-topic | Text | Casual chat in English |

### 6. 🇪🇸 SPANISH (Spanish Speaker role only)
| Channel | Type | Topic |
|---|---|---|
| #general | Text | Charlas generales en español |
| #dev-chat | Text | Discusión técnica en español |
| #off-topic | Text | Charlas casuales en español |

### 7. 🔊 VOICE (Member only — language neutral)
| Channel | Type |
|---|---|
| General VC | Voice |
| Project Stage | Voice |

### 8. 🛡️ STAFF (Admin only)
| Channel | Type |
|---|---|
| staff-chat | Text |
| mod-log | Text |
| bot-testing | Text |

## Roles

### Staff Roles (power)
| Role | Color | Permissions | Members |
|---|---|---|---|
| VantaDB (11) | `#e74c3c` | Administrator (managed bot) | 1 |
| Admin (10) | `#ff5500` | Administrator | 0 |
| Maintainer (9) | `#ea580c` | Kick, Manage Messages, Manage Roles, Voice perms | 0 |
| Contributor (8) | `#8b5cf6` | Send Messages, Embed, Attach, Voice perms | 0 |

### Progression Roles (earned)
| Role | Position | Color | Permissions |
|---|---|---|---|
| Member (7) | 7 | `#6b7280` | ViewChannel, SendMessages, Embed, Attach, Voice |
| New Member (6) | 6 | `#374151` | ViewChannel only |

### Identity Roles (self-assign via reaction roles)
| Role | Position | Color | Permissions |
|---|---|---|---|
| English Speaker (5) | 5 | `#3b82f6` | ViewChannel (for ENGLISH category access) |
| Spanish Speaker (4) | 4 | `#eab308` | ViewChannel (for SPANISH category access) |
| Rustacean (3) | 3 | `#e0e0e0` | None (cosmetic) |
| Pythonista (2) | 2 | `#e0e0e0` | None (cosmetic) |
| TypeScript (1) | 1 | `#e0e0e0` | None (cosmetic) |
| AI/ML | — | `#e0e0e0` | None (cosmetic) |

### System Roles
| Role | Position | Purpose |
|---|---|---|
| carl-bot (8) | 8 | Bot automation (must be above roles it manages) |
| @everyone (0) | 0 | Base permissions |

## Permission Architecture

### Category Permissions

| Category | @everyone | Member | English Speaker | Spanish Speaker | Admin |
|---|---|---|---|---|---|
| 📋 INFO | ✅ View | ✅ View | ✅ View | ✅ View | ✅ View |
| 👋 WELCOME | ✅ View | ✅ View | ✅ View | ✅ View | ✅ View |
| 💬 COMMUNITY | ❌ Deny | ✅ Allow | ✅ Allow | ✅ Allow | ✅ (Admin) |
| 🛠️ DEV | ❌ Deny | ✅ Allow | ✅ Allow | ✅ Allow | ✅ (Admin) |
| 🇬🇧 ENGLISH | ❌ Deny | ❌ Deny | ✅ Allow | ❌ | ✅ (Admin) |
| 🇪🇸 SPANISH | ❌ Deny | ❌ Deny | ❌ | ✅ Allow | ✅ (Admin) |
| 🔊 VOICE | ❌ Deny | ✅ Allow | ✅ Allow | ✅ Allow | ✅ (Admin) |
| 🛡️ STAFF | ❌ Deny | ❌ | ❌ | ❌ | ✅ Allow |

### Special Channel Overrides

| Channel | Override |
|---|---|
| #📜rules | @everyone ❌ SendMessages |
| #announcements | @everyone ❌ SendMessages |
| staff-chat | @everyone ❌ ViewChannel |
| mod-log | @everyone ❌ ViewChannel |
| bot-testing | @everyone ❌ ViewChannel |

## Server Settings

| Setting | Value |
|---|---|
| Verification Level | 1 (LOW — verified email) |
| Default Notifications | @mentions only |
| Explicit Media Filter | Scan all media |
| AFK Channel | General VC (5 min) |
| Server Widget | Enabled (channel: #announcements) |
| System Messages | All enabled (#👋introductions) |

## Bot Stack

| Bot | Purpose | Status |
|---|---|---|
| VantaDB (custom) | Server management via API | Active |
| Carl-bot | Reaction roles, autorole, logging, moderation, welcome messages | Installed — needs dashboard config |

## Integrations

| Integration | Events | Channel |
|---|---|---|
| GitHub Webhook | push, pull_request, issues, release | #announcements |
