# Bilingual Strategy — English / Spanish

## Architecture: Option A — Separate Language Channels

The server uses **separate channels per language** within dedicated categories.

### How It Works

1. **New members** arrive and see only 📋 INFO and 👋 WELCOME (shared English content)
2. **Community Onboarding** asks: *"What language do you prefer?"*
3. They pick **English** or **Spanish**, which auto-assigns the corresponding role
4. The role grants access to that language's category with dedicated channels
5. **Shared channels** (#general-chat, #dev-chat, etc.) remain language-neutral for cross-language interaction
6. **Carl-bot reaction roles** allow changing language at any time in #🎭roles

### Channel Structure

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

### Setup Requirements (manual via Carl-bot dashboard)

1. Create reaction roles in #🎭roles for English/Spanish
2. Configure Community Onboarding with language selection question
3. Set up autorole to grant Member role after verification
4. Optional: Set up auto-translation bot (Kiki/TradLinker) if cross-language conversation is desired in shared channels

### Roles

| Role | Color | Purpose |
|---|---|---|
| English Speaker | `#3b82f6` (blue) | Grants access to 🇬🇧 ENGLISH category |
| Spanish Speaker | `#eab308` (yellow/gold) | Grants access to 🇪🇸 SPANISH category |

### New Member Flow

1. User joins via invite
2. Accepts Membership Screening (6 rules)
3. Sees Welcome Screen with 5 channels
4. Completes Community Onboarding — picks language
5. Gets English Speaker or Spanish Speaker role + sees their channels
6. Can also pick language/tech roles in #🎭roles
7. Carl-bot autorole promotes to Member after N hours → unlocks COMMUNITY + DEV + VOICE

### Future Expansion

To add more languages (e.g., French, German, Japanese):
1. Create new role (e.g., "French Speaker")
2. Create new category (🇫🇷 FRENCH) with channels
3. Add reaction role in #🎭roles
4. Update Community Onboarding with new language option
