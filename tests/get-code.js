import fs from 'fs';
import { chromium } from 'playwright';
import path from 'path';

function parseArg(name, def) {
  const arg = process.argv.find(a => a.startsWith(`--${name}=`));
  if (!arg) return def;
  return arg.split('=')[1];
}

async function main() {
  const homeserver = parseArg('homeserver', 'https://test.aaca.eu.org');
  const redirectUrl = parseArg('redirectUrl', 'https://fluffychat.im/web/%23/home');
  const headlessArg = parseArg('headless', 'false');
  const headless = headlessArg !== 'true' ? false : true;
  const stealthArg = parseArg('stealth', 'true');
  const stealth = stealthArg === 'true';
  const noEncodeRedirect = parseArg('noEncodeRedirect', 'false') === 'true';
  const timeoutMs = parseInt(parseArg('timeout', '180000'), 10);

  const browser = await chromium.launch({ headless: !headless ? false : true });
  const storagePath = parseArg('storage', path.resolve(process.cwd(), 'playwright-storage.json'));
  let context;
  if (fs.existsSync(storagePath)) {
    context = await browser.newContext({ storageState: storagePath });
  } else {
    context = await browser.newContext();
  }
  const page = await context.newPage();

  if (stealth) {
    await page.addInitScript(() => {
      try { Object.defineProperty(navigator, 'webdriver', { get: () => false, configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'], configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'plugins', { get: () => [1,2,3,4,5], configurable: true }); } catch (e) {}
      try { window.chrome = window.chrome || { runtime: {} }; } catch (e) {}
    });
    console.log('Stealth enabled');
  }

  const redirectParam = noEncodeRedirect ? redirectUrl : encodeURIComponent(redirectUrl);
  const ssoRedirect = `${homeserver.replace(/\/+$/,'')}/_matrix/client/v3/login/sso/redirect?redirectUrl=${redirectParam}`;
  console.log('Opening SSO redirect URL:', ssoRedirect);

  // navigate (don't wait too long for networkidle because provider may redirect quickly)
  try {
    await page.goto(ssoRedirect, { waitUntil: 'load', timeout: 30000 });
  } catch (e) {
    console.log('Initial navigation error (may be normal):', e.message);
  }

  console.log(`Waiting for callback URL (timeout ${timeoutMs}ms). Complete any interactive challenge in the opened browser.`);

  let callbackUrl = null;
  try {
    await page.waitForURL('**/login/sso/callback**', { timeout: timeoutMs });
    callbackUrl = page.url();
  } catch (e) {
    // fallback: check last response/request URLs
    console.log('waitForURL timed out; scanning requests for callback...');
    const requests = page.context().pages().flatMap(p => p.requests ? p.requests() : []);
    for (const req of requests) {
      if (req.url().includes('/_matrix/client/v3/login/sso/callback')) {
        callbackUrl = req.url();
        break;
      }
    }
  }

  if (!callbackUrl) {
    console.error('Callback URL not detected within timeout');
    await browser.close();
    process.exit(2);
  }

  console.log('Detected callback URL:', callbackUrl);

  const u = new URL(callbackUrl);
  const code = u.searchParams.get('code');
  const state = u.searchParams.get('state');

  const out = { capturedAt: new Date().toISOString(), callbackUrl, code, state };
  fs.writeFileSync('last_code.json', JSON.stringify(out, null, 2));
  console.log(JSON.stringify(out, null, 2));

  // save storage state
  try {
    await context.storageState({ path: storagePath });
    console.log('Saved storage state to', storagePath);
  } catch (e) {
    console.warn('Failed to save storage state:', e && e.message);
  }

  await browser.close();
}

main().catch(err => {
  console.error('Script error:', err);
  process.exit(2);
});
