---
title: Pending Tasks — VantaDB Discord
kind: howto
status: active
description: "URL: https://dash.carl.gg/"
tags: [vantadb, discord, todo]
---

# Pending Tasks — VantaDB Discord

## 🔴 Critical (do ASAP)

### 1. Configure Carl-bot via Dashboard
**URL:** https://dash.carl.gg/
**Action:** Log in with Discord, select VantaDB Community

- [ ] **Reaction Roles** → Create message in #🎭roles
  - English Speaker (🇬🇧)
  - Spanish Speaker (🇪🇸)
  - Rustacean (🦀)
  - Pythonista (🐍)
  - TypeScript (🔷)
  - AI/ML (🤖)
  - Notification: News Ping (📢)
  - Notification: Release Ping (🚀)
- [ ] **Autorole** → Auto-assign New Member to new joiners, auto-promote to Member after N hours/days
- [ ] **Logging** → Send all logs to #mod-log (message delete, member join/leave, bans, kicks, role changes, voice events)
- [ ] **Welcome DM** → Automatic DM to new members:
  > Welcome to VantaDB Community! 👋
  > 
  > Start here:
  > 1. Read the rules: #📜rules
  > 2. Pick your language and roles: #🎭roles
  > 3. Ask in #🌐general or #❓help

### 2. Community Onboarding
**Path:** Server Settings → Community → Onboarding
- [ ] Enable onboarding
- [ ] Set default channels: #📜rules, #📢announcements, #🎭roles, #🌐general, #💻dev-chat
- [ ] Create question: *"What language do you prefer?"* (Single choice)
  - Option 1: English → assign English Speaker role → show 🇬🇧 ENGLISH channels
  - Option 2: Spanish → assign Spanish Speaker role → show 🇪🇸 SPANISH channels
- [ ] Set to **Advanced Mode** for granular channel access
- [ ] Test with alt account to confirm flow works

### 3. Server Banner (requires 2 boosts)
- [ ] Boost server ×2 ($10/month or 2 Nitro subscribers)
- [ ] Upload banner image (`docs/discord/assets/banner.png`)
- [ ] Update server description if needed

## 🟡 Important

### 4. Set up AutoMod
**Path:** Server Settings → Safety Setup
- [ ] Enable **Spam Detection** (message rate limits)
- [ ] Enable **Mass Mention** (>5 mentions → warn)
- [ ] Enable **Discord Invite Links** → delete
- [ ] Add custom blocked words (slurs, toxicity)
- [ ] Whitelist staff roles

### 5. Stickers & Emojis
- [ ] Upload VantaDB logo as server emoji (`:vantadb:`) — requires Discord UI
- [ ] Create custom stickers (VantaDB mascot, reactions) — requires Discord UI
- [ ] Upload standard utility emojis (check, alert, bug, etc.) — requires Discord UI

### 6. Forum Content Seeds
- [x] #❓help — 3 FAQ threads seeded (install, DLL troubleshooting, namespace setup)
- [x] #🌐showcase — 2 showcases seeded (LangGraph, AutoGen)
- [x] #💡ideas — 2 idea threads seeded (auto-embedding, PQ)
- [x] #🐛bug-reports — Bug report template posted

### 7. Scheduled Messages (Carl-bot)
- [ ] Weekly dev update reminder
- [ ] Monthly community events
- [ ] Release day announcements

## 🟢 Nice to Have

### 8. Ticketing System
- [ ] Install Ticket Tool (https://tickettool.xyz) or Helper.gg — requires bot auth, not API
- [ ] Configure for support requests
- [ ] Create ticket category/archive

### 9. Stage Channel
- [x] Create Stage channel (`#🎤 Stage`) — voice channel created
- [ ] Convert to Stage channel type (requires Discord UI or Community tier)
- [ ] Schedule first event (e.g., "VantaDB v0.4 Release Party")

### 10. Server Discovery (1000+ members)
- [ ] Meet 1000 member + 8 week requirements
- [ ] Enable in Community Settings
- [ ] Write compelling short description for discovery listing

### 11. Canny.io Roadmap
- [ ] Set up Canny board for public feature voting
- [ ] Integrate with Discord via webhook

## Marketing Checklist

- [ ] Add Discord badge to GitHub README
  ```markdown
  [![Discord](https://img.shields.io/discord/1522793011094749204)](https://discord.gg/g8nqB3NtXt)
  ```
- [ ] List on Disboard.org (bump every 2h)
- [ ] Post in relevant Reddit communities
- [ ] Add to VantaDB website footer
- [ ] Cross-promote with partner communities (Qdrant, Pinecone, Weaviate)

## Assets Pending
- [ ] **Server Banner** — Design dark tech banner (960×540) with VantaDB logo and tagline
- [ ] **Emoji Pack** — VantaDB logo, checkmark, alert, bug, sparkle, rocket
- [ ] **Sticker Pack** — Greeting, thank you, working on it, celebrating
