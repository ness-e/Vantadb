#!/usr/bin/env python3
"""
Visual inspection of VantaDB web app using Playwright
"""

import asyncio
import json
import os
from pathlib import Path

SCREENSHOTS_DIR = Path("visual-inspection")
SCREENSHOTS_DIR.mkdir(exist_ok=True)

async def inspect_page(page, name, url, wait_for="networkidle"):
    print("\nInspecting: " + name + " (" + url + ")")
    try:
        await page.goto(url, wait_until=wait_for, timeout=30000)
        await page.wait_for_timeout(2000)

        screenshot_path = Path("visual-inspection") / (name + ".png")
        await page.screenshot(path=str(screenshot_path), full_page=True)
        print("Screenshot saved: " + str(screenshot_path))

        viewport = page.viewport_size
        print("Viewport: " + str(viewport['width']) + "x" + str(viewport['height']))

        errors = []
        page.on("console", lambda msg: errors.append(msg.text) if msg.type == "error" else None)

        network_errors = []
        page.on("response", lambda resp: network_errors.append(str(resp.status) + " " + resp.url) if resp.status >= 400 else None)

        await page.wait_for_timeout(1000)

        if errors:
            print("Console errors:", errors)
        else:
            print("No console errors")

        if network_errors:
            print("Network errors:", network_errors)
        else:
            print("No network errors")

        # Check for overlapping elements
        overlapping = await page.evaluate("""() => {
            const elements = Array.from(document.querySelectorAll('*'));
            const issues = [];
            for (let i = 0; i < elements.length; i++) {
                const rect1 = elements[i].getBoundingClientRect();
                if (rect1.width === 0 || rect1.height === 0) continue;
                for (let j = i + 1; j < elements.length; j++) {
                    const rect2 = elements[j].getBoundingClientRect();
                    if (rect2.width === 0 || rect2.height === 0) continue;
                    const overlapX = Math.max(0, Math.min(rect1.right, rect2.right) - Math.max(rect1.left, rect2.left));
                    const overlapY = Math.max(0, Math.min(rect1.bottom, rect2.bottom) - Math.max(rect1.top, rect2.top));
                    if (overlapX > 10 && overlapY > 10) {
                        const style1 = window.getComputedStyle(elements[i]);
                        const style2 = window.getComputedStyle(elements[j]);
                        if (style1.zIndex !== style2.zIndex || style1.position === 'fixed' || style2.position === 'fixed') {
                            // Safely get className (handle SVG elements which have className.baseVal)
                            const getClassName = (el) => {
                                const cn = el.className;
                                return typeof cn === 'string' ? cn : (cn?.baseVal || '');
                            };
                            issues.push({
                                el1: elements[i].tagName + (elements[i].className ? '.' + (typeof elements[i].className === 'string' ? elements[i].className : (elements[i].className?.baseVal || '')).split(' ')[0] : ''),
                                el2: elements[j].tagName + (elements[j].className ? '.' + (typeof elements[j].className === 'string' ? elements[j].className : (elements[j].className?.baseVal || '')).split(' ')[0] : ''),
                                overlapX, overlapY
                            });
                        }
                    }
                }
            }
            return issues;
        }"""
        )

        if overlapping:
            print("Potential overlapping elements:", overlapping[:5])

        # Check for text clipping
        text_clipping = await page.evaluate("""() => {
            const issues = [];
            const textElements = document.querySelectorAll('p, h1, h2, h3, h4, h5, h6, span, a, button, label');
            textElements.forEach(el => {
                const rect = el.getBoundingClientRect();
                if (rect.width > 0 && rect.height > 0) {
                    const scrollWidth = el.scrollWidth;
                    const clientWidth = el.clientWidth;
                    if (scrollWidth > clientWidth + 5) {
                        issues.push({
                            tag: el.tagName,
                            class: el.className,
                            text: el.textContent?.substring(0, 50),
                            overflow: scrollWidth - clientWidth
                        });
                    }
                });
            });
            return issues;
        })"""
        )

        if text_clipping:
            print("Text clipping detected:", text_clipping[:5])

        # Check for images without alt
        missing_alt = await page.evaluate("""() => {
            const images = document.querySelectorAll('img:not([alt])');
            return Array.from(images).map(img => ({
                src: img.src,
                alt: img.alt,
                className: img.className
            }));
        })"""
        )

        if missing_alt:
            print("Images missing alt:", missing_alt[:5])

        # Check for buttons/links without accessible names
        missing_aria = await page.evaluate("""() => {
            const elements = document.querySelectorAll('button:not([aria-label]):not([aria-labelledby]), a:not([aria-label]):not([aria-labelledby])');
            return Array.from(elements).filter(el => !el.textContent?.trim()).map(el => ({
                tag: el.tagName,
                className: el.className,
                href: el.href
            }));
        })"""
        )

        if missing_aria:
            print("Missing accessible names:", missing_aria[:5])

        # Check CLS
        cls = await page.evaluate("""() => {
            return new Promise(resolve => {
                let clsValue = 0;
                const observer = new PerformanceObserver(list => {
                    for (const entry of list.getEntries()) {
                        if (!entry.hadRecentInput) {
                            clsValue += entry.value;
                        }
                    }
                });
                observer.observe({ type: 'layout-shift', buffered: true });
                setTimeout(() => resolve(clsValue), 3000);
            });
        })"""
        )

        if cls > 0.1:
            print("CLS (Cumulative Layout Shift): " + str(round(cls, 4)) + " - above good threshold (0.1)")
        else:
            print("CLS: " + str(round(cls, 4)))

        return {
            "name": name,
            "url": url,
            "screenshot": "visual-inspection/" + name + ".png",
            "errors": [],
            "networkErrors": [],
            "overlapping": [],
            "textClipping": [],
            "missingAlt": [],
            "missingAria": [],
            "cls": cls
        }
    except Exception as e:
        print("Error inspecting " + name + ": " + str(e))
        return {"name": name, "url": url, "error": str(e)}


async def main():
    print("Starting visual inspection of VantaDB web app (production)...")

    from playwright.async_api import async_playwright
    import json
    from pathlib import Path

    screenshots_dir = Path("visual-inspection")
    screenshots_dir.mkdir(exist_ok=True)

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

        page.on("console", lambda msg: print("[CONSOLE ERROR] " + msg.text) if msg.type == "error" else None)
        page.on("pageerror", lambda error: print("[PAGE ERROR] " + error.message))

        results = []

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

            page.on("console", lambda msg: print("[CONSOLE ERROR] " + msg.text) if msg.type == "error" else None)
            page.on("pageerror", lambda error: print("[PAGE ERROR] " + error.message))

            results = []

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

            for page_info in pages:
                result = await inspect_page(page, page_info["name"], page_info["url"])
                results.append(result)

                await page.goto("https://vantadb.vercel.app", wait_until="networkidle")
                await page.wait_for_timeout(1000)

            # Save results
            report_path = Path("visual-inspection/inspection-report.json")
            with open(report_path, "w") as f:
                json.dump(results, f, indent=2)
            print("\nReport saved: " + str(report_path))

            # Summary
            print("\nINSPECTION SUMMARY:")
            print("=======================")
            for r in results:
                if "error" in r:
                    print("FAIL " + r["name"] + ": " + r["error"])
                else:
                    issues = []
                    if r.get("errors"): issues.append(str(len(r["errors"])) + " console errors")
                    if r.get("networkErrors"): issues.append(str(len(r["networkErrors"])) + " network errors")
                    if r.get("overlapping"): issues.append(str(len(r["overlapping"])) + " overlapping elements")
                    if r.get("textClipping"): issues.append(str(len(r["textClipping"])) + " text clipping")
                    if r.get("missingAlt"): issues.append(str(len(r["missingAlt"])) + " missing alt")
                    if r.get("missingAria"): issues.append(str(len(r["missingAria"])) + " missing aria")
                    if r.get("cls", 0) > 0.1: issues.append("CLS " + str(round(r["cls"], 4)))

                    if not issues:
                        print("OK " + r["name"] + ": Clean")
                    else:
                        print("ISSUES " + r["name"] + ": " + ", ".join(issues))

            await browser.close()
            print("\nVisual inspection complete!")


if __name__ == "__main__":
    import asyncio
    asyncio.run(main())