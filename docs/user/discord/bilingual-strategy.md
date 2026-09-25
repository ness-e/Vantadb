---
title: "Bilingual Strategy — Single-Channel Model"
type: discord
status: active
tags: [vantadb, discord, bilingual]
last_reviewed: 2026-07-21
---

# Bilingual Strategy — English / Spanish

## Model: Single-Channel Bilingual (current)

The server uses a **single set of shared channels** where English and Spanish coexist. No language-gated categories.

### How It Works

1. **All members** share the same channels: #🌐general, #💻dev-chat, #🗣️off-topic
2. **Anyone** can write in English or Spanish — both are welcome
3. **Language roles** (English Speaker 🇬🇧, Spanish Speaker 🇪🇸) exist as **identity badges only** — they don't gate access to any channel
4. **No translation bot** is installed — members self-manage language choice
5. **Future expansion** to separate channels is possible if the server grows past the point where a single channel becomes noisy

### Why Single-Channel?

| Razón | Detalle |
|-------|---------|
| **Simplicidad** | 3 members, 1 active. No necesita separación. |
| **Visibilidad** | Todo el contenido visible para todos sin cambiar de categoría. |
| **Bajo overhead** | Sin bots de traducción, sin permisos por rol, sin onboarding complejo. |
| **Fomenta bilingualismo** | Developers ven ambos idiomas naturalmente. |

### If the server grows (future option)

If member count increases and language segregation becomes necessary, the setup would migrate to:

```
🇬🇧 ENGLISH (requires English Speaker role)
├── #general
├── #dev-chat
└── #off-topic

🇪🇸 SPANISH (requires Spanish Speaker role)
├── #general
├── #dev-chat
└── #off-topic
```

This migration requires:
1. Changing language roles from cosmetic to permission-gated
2. Restricting @everyone ViewChannel on current shared channels
3. Creating new EN/ES category permissions

### Roles

| Role | Color | Purpose |
|------|-------|---------|
| English Speaker | `#3b82f6` (blue) | Identity badge — no channel gating |
| Spanish Speaker | `#eab308` (yellow/gold) | Identity badge — no channel gating |

### New Member Flow

1. User joins via invite
2. Accepts Membership Screening (6 rules)
3. Sees Welcome Screen with 5 channels (#📜rules, #🎭roles, #📢announcements, #🌐general, #💻dev-chat)
4. Picks language and tech roles in #🎭roles (via Carl-bot reaction roles)
5. Carl-bot autorole promotes to Member after N hours → unlocks full channel access

### Future Expansion

To add more languages (e.g., French, German, Japanese):
1. Create new role (e.g., "French Speaker")
2. If using separate channels: create new category with channel copies + permission gates
3. If staying single-channel: just add the role as cosmetic identity
4. Add reaction role in #🎭roles
5. Update Community Onboarding with new language option
