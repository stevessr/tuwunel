import fs from 'fs';
import { chromium } from 'playwright';
import path from 'path';

// Usage:
//  node tests/capture-login.js [--homeserver=https://test.aaca.eu.org] [--redirectUrl=https://fluffychat.im/web/%23/home] [--headless=false]

function parseArg(name, def) {
  const arg = process.argv.find(a => a.startsWith(`--${name}=`));
  if (!arg) return def;
  return arg.split('=')[1];
}

async function main() {
  const homeserver = parseArg('homeserver', null);
  const redirectUrl = parseArg('redirectUrl', 'https://fluffychat.im/web/%23/home');
  const noEncodeRedirect = parseArg('noEncodeRedirect', 'false') === 'true';
  const headlessArg = parseArg('headless', 'true');
  const headless = headlessArg !== 'false';
  const stealthArg = parseArg('stealth', 'false');
  const stealth = stealthArg === 'true';

  const browser = await chromium.launch({ headless });

  const storagePath = parseArg('storage', path.resolve(process.cwd(), 'playwright-storage.json'));
  let context;
  // stealth mode: set common Chrome UA + languages + viewport to reduce automation fingerprint
  if (stealth) {
    const userAgent = parseArg('userAgent', 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36');
    const contextOpts = { userAgent, locale: 'en-US', viewport: { width: 1280, height: 800 } };
    if (fs.existsSync(storagePath)) {
      console.log('Loading storage state from', storagePath);
      context = await browser.newContext({ ...contextOpts, storageState: storagePath });
    } else {
      context = await browser.newContext(contextOpts);
    }
  } else {
    if (fs.existsSync(storagePath)) {
      console.log('Loading storage state from', storagePath);
      context = await browser.newContext({ storageState: storagePath });
    } else {
      context = await browser.newContext();
    }
  }
  const page = await context.newPage();

  // If stealth mode is requested, inject scripts to reduce automation fingerprints
  if (stealth) {
    await page.addInitScript(() => {
      // navigator.webdriver
      try { Object.defineProperty(navigator, 'webdriver', { get: () => false, configurable: true }); } catch (e) {}
      // languages
      try { Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'], configurable: true }); } catch (e) {}
      // plugins
      try { Object.defineProperty(navigator, 'plugins', { get: () => [1,2,3,4,5], configurable: true }); } catch (e) {}
      // window.chrome
      try { window.chrome = window.chrome || { runtime: {} }; } catch (e) {}
      // permissions
      try {
        const _query = window.navigator.permissions.query;
        window.navigator.permissions.query = (parameters) => (
          parameters && parameters.name === 'notifications'
            ? Promise.resolve({ state: Notification.permission })
            : _query(parameters)
        );
      } catch (e) {}
    });
    console.log('Stealth mode enabled: injected anti-detection scripts and set common UA/locale/viewport');
  }

  const captures = [];

  page.on('request', request => {
    captures.push({
      type: 'request',
      url: request.url(),
      method: request.method(),
      headers: request.headers(),
      postData: request.postData(),
      timestamp: Date.now()
    });
    console.log('REQ', request.method(), request.url());
  });

  page.on('response', async response => {
    let body = null;
    try {
      body = await response.text();
    } catch (e) {
      body = `<<unable to read: ${e.message}>>`;
    }
    captures.push({
      type: 'response',
      url: response.url(),
      status: response.status(),
      headers: response.headers(),
      body,
      timestamp: Date.now()
    });
    console.log('RES', response.status(), response.url());
  });

  // longer timeout for interactive flows
  const timeoutMs = 120000;
  const waitPromise = new Promise((resolve) => {
    const start = Date.now();
    const check = () => {
      // look for POST to /_matrix/client/v3/login in captures
      const found = captures.find(c => c.type === 'request' && c.method === 'POST' && c.url.includes('/_matrix/client/v3/login'));
      if (found) return resolve(found);
      if (Date.now() - start > timeoutMs) return resolve(null);
      setTimeout(check, 500);
    };
    check();
  });

  if (homeserver) {
    // Try opening the homeserver's SSO redirect endpoint to trigger the flow
    const redirectParam = noEncodeRedirect ? redirectUrl : encodeURIComponent(redirectUrl);
    const url = `${homeserver.replace(/\/+$/,'')}/_matrix/client/v3/login/sso/redirect?redirectUrl=${redirectParam}`;
    console.log('Opening homeserver SSO redirect URL:', url);
    try {
      await page.goto(url, { waitUntil: 'networkidle' });
    } catch (e) {
      console.log('Navigation error (this can be normal during external SSO redirect):', e.message);
    }
  } else {
    console.log('Opening https://fluffychat.im/web/#/home');
    await page.goto('https://fluffychat.im/web/#/home', { waitUntil: 'networkidle' });
  }

  let foundReq = await waitPromise;

  const out = {
    meta: { capturedAt: new Date().toISOString(), target: homeserver ? 'homeserver-sso-redirect' : 'https://fluffychat.im/web/#/home', homeserver, redirectUrl, headless },
    captures
  };
  fs.writeFileSync('captures.json', JSON.stringify(out, null, 2));

  if (!foundReq && !headless) {
    console.log('\n未检测到 POST /_matrix/client/v3/login；进入交互等待。请在打开的浏览器中完成登录/通过 Cloudflare challenge。完成后在此终端按回车继续并保存浏览器状态（或等待超时）。');
    await new Promise(resolve => {
      process.stdin.resume();
      process.stdin.once('data', () => {
        process.stdin.pause();
        resolve();
      });
    });
    // check again
    foundReq = captures.find(c => c.type === 'request' && c.method === 'POST' && c.url.includes('/_matrix/client/v3/login')) || null;
  }

  if (foundReq) {
    console.log('\nFound POST /_matrix/client/v3/login request:');
    console.log(JSON.stringify(foundReq, null, 2));
  } else {
    console.log('\nPOST /_matrix/client/v3/login not seen within timeout');
  }
  // persist storage state so manual login / cookies / auth survive next run
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
