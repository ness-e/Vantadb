const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  
  await page.goto('http://127.0.0.1:3000', { waitUntil: 'networkidle', timeout: 30000 });
  
  // Wait for animations to settle
  await page.waitForTimeout(2000);
  
  // Full page screenshot
  await page.screenshot({ path: 'before-home-full.png', fullPage: true });
  
  // TrustBar specific
  const trustBar = page.locator('section[aria-label*="Ecosistema"]').first();
  if (await trustBar.count()) {
    await trustBar.screenshot({ path: 'before-trustbar.png' });
  }
  
  // Hero specific
  const hero = page.locator('section[aria-label*="VantaDB"]').first();
  if (await hero.count()) {
    await hero.screenshot({ path: 'before-hero.png' });
  }
  
  await browser.close();
  console.log('Screenshots saved');
})();