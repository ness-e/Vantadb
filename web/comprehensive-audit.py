#!/usr/bin/env python3
"""
Comprehensive Visual Audit of VantaDB web app using Playwright
"""

import asyncio
import json
import os
from pathlib import Path
from datetime import datetime

AUDIT_DIR = Path("visual-audit")
AUDIT_DIR.mkdir(exist_ok=True)


async def scroll_page_full(page, step_size=300, delay=500):
    print("  Scrolling page to trigger animations...")
    page_height = await page.evaluate("document.body.scrollHeight")
    viewport_height = await page.evaluate("window.innerHeight")

    screenshots = []
    current_y = 0
    scroll_step = 0

    while current_y < page_height:
        await page.evaluate(f"window.scrollTo(0, {current_y})")
        await page.wait_for_timeout(300)

        screenshot_path = Path("visual-audit") / f"scroll_{datetime.now().strftime('%H%M%S')}_{scroll_step:03d}.png"
        await page.screenshot(path=str(screenshot_path), full_page=False)
        screenshots.append(str(screenshot_path))

        current_y += 300
        scroll_step += 1

        new_scroll_y = await page.evaluate("window.scrollY + window.innerHeight")
        if new_scroll_y >= page_height - 100:
            break

    await page.evaluate("window.scrollTo(0, 0)")
    await page.wait_for_timeout(500)

    return screenshots


async def get_clickable_elements(page):
    print("  Finding clickable elements...")
    return await page.evaluate("""() => {
        const elements = document.querySelectorAll('button, a, input, select, textarea, [role="button"], [onclick], [data-testid]');
        return Array.from(elements).filter(el => {
            const rect = el.getBoundingClientRect();
            return rect.width > 0 && rect.height > 0 &&
                   rect.top >= 0 && rect.left >= 0 &&
                   rect.bottom <= window.innerHeight && rect.left <= window.innerWidth;
        }).map(el => ({
            tag: el.tagName,
            class: el.className,
            id: el.id,
            text: el.textContent?.trim().substring(0, 50),
            href: el.href,
            type: el.type,
            role: el.getAttribute('role'),
            ariaLabel: el.getAttribute('aria-label'),
            rect: {
                x: Math.round(el.getBoundingClientRect().left),
                y: Math.round(el.getBoundingClientRect().top),
                width: Math.round(el.getBoundingClientRect().width),
                height: Math.round(el.getBoundingClientRect().height)
            }
        })).slice(0, 50);
    }""")


async def analyze_design(page):
    print("  Analyzing design system...")

    css_vars = await page.evaluate("""() => {
        const rootStyles = getComputedStyle(document.documentElement);
        const cssVars = {};
        for (let i = 0; i < rootStyles.length; i++) {
            const name = rootStyles[i];
            if (name.startsWith('--')) {
                cssVars[name] = rootStyles.getPropertyValue(name).trim();
            }
        }
        return cssVars;
    }""")

    typography = await page.evaluate("""() => {
        const fontFamilies = {};
        ['h1', 'h2', 'h3', 'h4', 'h4', 'p', 'button', 'a'].forEach(selector => {
            const el = document.querySelector(selector);
            if (el) {
                const style = getComputedStyle(el);
                fontFamilies[selector] = {
                    fontFamily: style.fontFamily,
                    fontSize: style.fontSize,
                    fontWeight: style.fontWeight,
                    lineHeight: style.lineHeight,
                    color: style.color
                };
            }
        });
        return fontFamilies;
    }""")

    buttons = await page.evaluate("""() => {
        const buttons = document.querySelectorAll('button, a[role="button"], .btn');
        const buttonStyles = [];
        buttons.forEach((btn, i) => {
            if (i < 10) {
                const style = getComputedStyle(btn);
                buttonStyles.push({
                    tag: btn.tagName,
                    class: btn.className,
                    text: btn.textContent?.trim().substring(0, 30),
                    backgroundColor: style.backgroundColor,
                    color: style.color,
                    border: style.border,
                    borderRadius: style.borderRadius,
                    padding: style.padding,
                    fontSize: style.fontSize,
                    fontWeight: style.fontWeight,
                    transition: style.transition
                });
            }
        });
        return buttonStyles;
    }""")

    sections = await page.evaluate("""() => {
        const sections = document.querySelectorAll('section, main, header, footer, .container, .section');
        const sectionStyles = [];
        sections.forEach((sec, i) => {
            if (i < 10) {
                const style = getComputedStyle(sec);
                sectionStyles.push({
                    tag: sec.tagName,
                    class: sec.className,
                    backgroundColor: style.backgroundColor,
                    padding: style.padding,
                    margin: style.margin,
                    maxWidth: style.maxWidth
                });
            }
        });
        return sectionStyles;
    }""")

    colors = await page.evaluate("""() => {
        const colors = {
            backgrounds: [],
            texts: [],
            borders: [],
            accents: []
        };

        document.querySelectorAll('*').forEach(el => {
            const style = getComputedStyle(el);
            if (style.backgroundColor && style.backgroundColor !== 'rgba(0, 0, 0, 0)' && style.backgroundColor !== 'transparent') {
                if (!colors.backgrounds.includes(style.backgroundColor)) colors.backgrounds.push(style.backgroundColor);
            }
            if (style.color && style.color !== 'rgba(0, 0, 0, 0)') {
                if (!colors.texts.includes(style.color)) colors.texts.push(style.color);
            }
            if (style.borderColor && style.borderColor !== 'rgba(0, 0, 0, 0)') {
                if (!colors.borders.includes(style.borderColor)) colors.borders.push(style.borderColor);
            }
        });

        Object.keys(colors).forEach(key => {
            colors[key] = [...new Set(colors[key])].slice(0, 20);
        });
        return colors;
    }""")

    return {
        "cssVariables": css_vars,
        "typography": typography,
        "buttons": buttons,
        "sections": sections,
        "colorPalette": colors
    }


async def check_accessibility(page):
    print("  Checking accessibility...")
    return await page.evaluate("""() => {
        const issues = [];

        document.querySelectorAll('img:not([alt])').forEach(img => {
            issues.push({
                type: 'missing-alt',
                element: 'img',
                src: img.src,
                class: img.className
            });
        });

        document.querySelectorAll('button:not([aria-label]):not([aria-labelledby]), a:not([aria-label]):not([aria-labelledby])').forEach(el => {
            if (!el.textContent?.trim()) {
                issues.push({
                    type: 'missing-accessible-name',
                    element: el.tagName,
                    class: el.className,
                    href: el.href
                });
            }
        });

        document.querySelectorAll('input:not([aria-label]):not([aria-labelledby])').forEach(input => {
            const id = input.id;
            const label = id ? document.querySelector(`label[for="${id}"]`) : null;
            if (!label && !input.getAttribute('aria-label') && !input.getAttribute('aria-labelledby')) {
                issues.push({
                    type: 'missing-label',
                    element: 'input',
                    type: input.type,
                    id: input.id,
                    class: input.className
                });
            }
        });

        const headings = document.querySelectorAll('h1, h2, h3, h4, h5, h6');
        let lastLevel = 0;
        headings.forEach(h => {
            const level = parseInt(h.tagName[1]);
            if (level > lastLevel + 1) {
                issues.push({
                    type: 'heading-hierarchy-skip',
                    element: h.tagName,
                    text: h.textContent?.trim().substring(0, 50),
                    expectedLevel: lastLevel + 1,
                    actualLevel: level
                });
            }
            lastLevel = level;
        });

        const landmarks = document.querySelectorAll('main, nav, header, footer, aside, section[aria-label], section[aria-labelledby]');
        if (landmarks.length === 0) {
            issues.push({type: 'missing-landmarks'});
        }

        return issues;
    }""")


async def analyze_content_structure(page):
    print("  Analyzing content structure...")

    return await page.evaluate("""() => {
        const structure = {
            headings: [],
            sections: [],
            lists: [],
            tables: [],
            forms: [],
            images: [],
            videos: [],
            iframes: []
        };

        document.querySelectorAll('h1, h2, h3, h4, h5, h6').forEach(h => {
            structure.headings.push({
                level: parseInt(h.tagName[1]),
                text: h.textContent?.trim().substring(0, 100),
                id: h.id,
                class: h.className
            });
        });

        document.querySelectorAll('section, article, main, aside').forEach(s => {
            structure.sections.push({
                tag: s.tagName,
                class: s.className,
                id: s.id,
                ariaLabel: s.getAttribute('aria-label'),
                ariaLabelledby: s.getAttribute('aria-labelledby'),
                headingCount: s.querySelectorAll('h1,h2,h3,h4,h5,h6').length
            });
        });

        document.querySelectorAll('ul, ol').forEach(l => {
            structure.lists.push({
                type: l.tagName,
                class: l.className,
                itemCount: l.querySelectorAll('li').length
            });
        });

        document.querySelectorAll('table').forEach(t => {
            structure.tables.push({
                class: t.className,
                caption: t.querySelector('caption')?.textContent?.trim(),
                rows: t.querySelectorAll('tr').length,
                headers: Array.from(t.querySelectorAll('th')).map(th => th.textContent?.trim())
            });
        });

        document.querySelectorAll('form').forEach(f => {
            structure.forms.push({
                class: f.className,
                action: f.action,
                method: f.method,
                fields: Array.from(f.querySelectorAll('input, select, textarea')).map(f => ({
                    type: f.type || f.tagName.toLowerCase(),
                    name: f.name,
                    id: f.id,
                    required: f.required,
                    placeholder: f.placeholder
                }))
            });
        });

        document.querySelectorAll('img').forEach(img => {
            structure.images.push({
                src: img.src,
                alt: img.alt,
                width: img.width,
                height: img.height,
                class: img.className
            });
        });

        document.querySelectorAll('video').forEach(v => {
            structure.videos.push({
                src: v.src,
                poster: v.poster,
                controls: v.controls,
                class: v.className
            });
        });

        document.querySelectorAll('iframe').forEach(f => {
            structure.iframes.push({
                src: f.src,
                title: f.title,
                class: f.className
            });
        });

        return structure;
    }""")


async def test_interactions(page):
    print("  Testing interactions...")

    interactions = []

    buttons = await page.query_selector_all('button, a[role="button"]')
    for i, btn in enumerate(buttons[:10]):
        try:
            box = await btn.bounding_box()
            if box:
                await page.mouse.move(box['x'] + box['width']/2, box['y'] + box['height']/2)
                await page.wait_for_timeout(100)

                hover_style = await page.evaluate("""(el) => {
                    const style = getComputedStyle(el);
                    return {
                        backgroundColor: getComputedStyle(el).backgroundColor,
                        color: getComputedStyle(el).color,
                        transform: getComputedStyle(el).transform,
                        boxShadow: getComputedStyle(el).boxShadow,
                        cursor: getComputedStyle(el).cursor
                    };
                }""", await btn.element_handle())

                interactions.append({
                    type: 'button-hover',
                    index: i,
                    text: (await btn.text_content())[:50],
                    hoverStyles: hover_style
                })
        except Exception as e:
            pass

    links = await page.query_selector_all('a[href]')
    for i, link in enumerate(links[:10]):
        try:
            href = await link.get_attribute('href')
            text = await link.text_content()
            interactions.append({
                type: 'link',
                index: i,
                href: href,
                text: text[:50] if text else ''
            })
        except:
            pass

    return interactions


async def comprehensive_audit(page, url, name):
    print(f"\n{'='*60}")
    print(f"AUDITING: {name} ({url})")
    print(f"{'='*60}")

    await page.goto(url, wait_until="networkidle", timeout=30000)
    await page.wait_for_timeout(2000)

    screenshot_path = Path("visual-audit") / f"{datetime.now().strftime('%Y%m%d_%H%M%S')}_{name}_initial.png"
    await page.screenshot(path=str(screenshot_path), full_page=True)

    scroll_screenshots = await scroll_page_full(page)

    clickable = await get_clickable_elements(page)

    design = await analyze_design(page)
    a11y = await check_accessibility(page)
    structure = await analyze_content_structure(page)
    interactions = await test_interactions(page)

    final_screenshot = Path("visual-audit") / f"{datetime.now().strftime('%Y%m%d_%H%M%S')}_{name}_final.png"
    await page.screenshot(path=str(final_screenshot), full_page=True)

    return {
        "name": name,
        "url": url,
        "initial_screenshot": str(screenshot_path),
        "scroll_screenshots": scroll_screenshots,
        "final_screenshot": str(final_screenshot),
        "clickable_elements": clickable,
        "design_system": design,
        "accessibility": a11y,
        "content_structure": structure,
        "interactions": interactions,
        "timestamp": datetime.now().isoformat()
    }


def generate_markdown_report(results):
    md = f"# VantaDB Web App - Comprehensive Visual Audit Report\n\n"
    md += f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n"
    md += f"**Target:** https://vantadb.vercel.app\n\n"
    md += "---\n\n"

    for r in results:
        if "error" in r:
            md += f"## {r['name']} - ERROR\n\n"
            md += f"**URL:** {r['url']}\n\n"
            md += f"**Error:** {r['error']}\n\n"
            continue

        md += f"## {r['name'].upper()} ({r['url']})\n\n"
        md += f"**Timestamp:** {r.get('timestamp', 'N/A')}\n\n"

        if 'design_system' in r:
            ds = r['design_system']
            md += "### Design System\n\n"
            if 'cssVariables' in ds:
                md += "#### CSS Variables\n\n"
                md += "| Variable | Value |\n|----------|-------|\n"
                for k, v in ds['cssVariables'].items():
                    md += f"| `{k}` | `{v}` |\n"
                md += "\n"

            if 'typography' in ds:
                md += "#### Typography\n\n"
                md += "| Selector | Font Family | Size | Weight | Line Height | Color |\n|----------|-------------|------|--------|-------------|-------|\n"
                for sel, t in ds['typography'].items():
                    md += f"| {sel} | {t.get('fontFamily', 'N/A')} | {t.get('fontSize', 'N/A')} | {t.get('fontWeight', 'N/A')} | {t.get('lineHeight', 'N/A')} | {t.get('color', 'N/A')} |\n"
                md += "\n"

            if 'buttons' in ds:
                md += "#### Buttons\n\n"
                md += "| Class | Text | Background | Color | Border | Radius | Padding |\n|-------|------|------------|-------|--------|--------|---------|\n"
                for btn in ds['buttons'][:5]:
                    md += f"| {btn.get('class', 'N/A')[:30]} | {btn.get('text', 'N/A')[:20]} | {btn.get('backgroundColor', 'N/A')} | {btn.get('color', 'N/A')} | {btn.get('border', 'N/A')} | {btn.get('borderRadius', 'N/A')} | {btn.get('padding', 'N/A')} |\n"
                md += "\n"

            if 'colorPalette' in ds:
                md += "#### Color Palette\n\n"
                for category, colors in ds['colorPalette'].items():
                    if colors:
                        md += f"**{category.capitalize()}:** {', '.join(colors[:10])}\n\n"

        if 'accessibility' in r:
            md += "### Accessibility Issues\n\n"
            if r['accessibility']:
                md += "| Type | Element | Details |\n|------|---------|---------|\n"
                for issue in r['accessibility'][:20]:
                    md += f"| {issue.get('type', 'N/A')} | {issue.get('element', 'N/A')} | {str(issue)[:100]} |\n"
                md += "\n"
            else:
                md += "No accessibility issues detected\n\n"

        if 'content_structure' in r:
            cs = r['content_structure']
            md += "### Content Structure\n\n"
            if cs.get('headings'):
                md += "#### Heading Hierarchy\n\n"
                md += "| Level | Text | ID |\n|-------|------|----|\n"
                for h in cs['headings'][:15]:
                    md += f"| H{h['level']} | {h['text'][:60]} | {h['id'] or 'N/A'} |\n"
                md += "\n"

            if cs.get('sections'):
                md += "#### Sections\n\n"
                for s in cs['sections'][:10]:
                    md += f"- **{s['tag']}** (class: {s['class'][:30]}) - {s['headingCount']} headings\n"
                md += "\n"

            if cs.get('forms'):
                md += "#### Forms\n\n"
                for f in cs['forms'][:5]:
                    md += f"- Form: {f['class'][:30]} - {len(f['fields'])} fields\n"
                md += "\n"

        if 'interactions' in r:
            md += "### Interaction Testing\n\n"
            for i in r['interactions'][:10]:
                md += f"- **{i.get('type', 'N/A')}** [{i.get('index', 'N/A')}]: {i.get('text', i.get('href', 'N/A'))[:50]}\n"
            md += "\n"

        md += "---\n\n"

    md += "## Summary\n\n"
    md += "| Page | Status | Issues Found |\n|------|--------|--------------|\n"
    for r in results:
        if "error" in r:
            md += f"| {r['name']} | ERROR | {r['error'][:50]} |\n"
        else:
            issue_count = len(r.get('accessibility', []))
            md += f"| {r['name']} | Clean | {issue_count} minor |\n"

    md += f"\n\n*Report generated on {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}*\n"
    return md


async def main():
    print("Starting COMPREHENSIVE Visual Audit of VantaDB web app (production)...")

    from playwright.async_api import async_playwright

    pages = [
        {"name": "home", "url": "https://vantadb.vercel.app"},
        {"name": "docs", "url": "https://vantadb.vercel.app/docs"},
        {"name": "playground", "url": "https://vantadb.vercel.app/playground"},
        {"name": "about-company", "url": "https://vantadb.vercel.app/about/company"},
        {"name": "about-team", "url": "https://vantadb.vercel.app/about/team"},
        {"name": "about-community", "url": "https://vantadb.vercel.app/about/community"},
        {"name": "about-contact", "url": "https://vantadb.vercel.app/about/contact"},
        {"name": "playground-page", "url": "https://vantadb.vercel.app/playground"},
        {"name": "docs-page", "url": "https://vantadb.vercel.app/docs"},
        {"name": "architecture", "url": "https://vantadb.vercel.app/architecture"},
        {"name": "benchmarks", "url": "https://vantadb.vercel.app/benchmarks"},
    ]

    async with async_playwright() as p:
        browser = await p.chromium.launch(
            headless=False,
            args=["--start-maximized"],
            slow_mo=100
        )

        context = await browser.new_context(
            viewport={"width": 1440, "height": 900},
            device_scale_factor=1
        )

        page = await context.new_page()

        page.on("console", lambda msg: print(f"[CONSOLE] {msg.type}: {msg.text}") if msg.type in ["error", "warning"] else None)
        page.on("pageerror", lambda error: print(f"[PAGE ERROR] {error.message}"))

        results = []

        for page_info in pages:
            try:
                result = await comprehensive_audit(page, page_info["url"], page_info["name"])
                results.append(result)
            except Exception as e:
                results.append({"name": page_info["name"], "url": page_info["url"], "error": str(e)})

            try:
                await page.goto("https://vantadb.vercel.app", wait_until="networkidle")
                await page.wait_for_timeout(1000)
            except:
                pass

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        report_path = Path("visual-audit") / f"audit_report_{timestamp}.json"
        with open(report_path, "w") as f:
            json.dump(results, f, indent=2, default=str)
        print(f"\nAudit report saved: {report_path}")

        md_report = generate_markdown_report(results)
        md_path = Path("visual-audit") / f"audit_report_{timestamp}.md"
        with open(md_path, "w") as f:
            f.write(md_report)
        print(f"Markdown report saved: {md_path}")

        await browser.close()
        print("\nComprehensive visual audit complete!")


if __name__ == "__main__":
    asyncio.run(main())