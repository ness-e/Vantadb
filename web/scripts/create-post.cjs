#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Extract the title from command line arguments
const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('\x1b[31mError: Provide a title for the blog post.\x1b[0m');
  console.log('Usage: npm run create-post "My Awesome Title"');
  process.exit(1);
}

const title = args[0];

// Convert title to a URL-friendly slug
const slug = title
  .toLowerCase()
  .replace(/[^a-z0-9]+/g, '-')
  .replace(/(^-|-$)/g, '');

const date = new Date().toISOString().split('T')[0];

const contentDir = path.join(__dirname, '..', 'content', 'blog');

// Ensure directory exists
if (!fs.existsSync(contentDir)) {
  fs.mkdirSync(contentDir, { recursive: true });
}

const filePath = path.join(contentDir, `${slug}.md`);

if (fs.existsSync(filePath)) {
  console.error(`\x1b[31mError: A post with the slug "${slug}" already exists.\x1b[0m`);
  process.exit(1);
}

const frontmatter = `---
title: "${title}"
date: "${date}"
description: "Escribe una descripción breve aquí (1-2 oraciones)."
author: "VantaDB Team"
tags: ["engineering"]
---

## Introducción

Empieza a escribir tu post aquí...
`;

fs.writeFileSync(filePath, frontmatter, 'utf8');

console.log(`\x1b[32mSuccess! Blog post created.\x1b[0m`);
console.log(`File: \x1b[36mweb/content/blog/${slug}.md\x1b[0m`);
console.log('You can now open the file and start writing.');
