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
  const redirectUrl = parseArg('redirectUrl', 'https://fluffychat.im/web/#/home');
  const noEncodeRedirect = parseArg('noEncodeRedirect', 'false') === 'true';
  const headlessArg = parseArg('headless', 'true');
  const headless = headlessArg !== 'false';
  const stealthArg = parseArg('stealth', 'false');
  const stealth = stealthArg === 'true';
  const prefillHome = parseArg('prefillHome', 'false') === 'true';

  // Add some launch args when stealth mode is requested to reduce automation flags
  const launchArgs = [];
  if (stealth) {
    launchArgs.push('--disable-blink-features=AutomationControlled');
  }
  const chromePath = parseArg('chromePath', null);
  const userDataDir = parseArg('userDataDir', null);
  const launchOpts = { headless, args: launchArgs };
  if (chromePath) launchOpts.executablePath = chromePath;

  let browser = null;
  let persistentContext = null;
  let context = null;
  const storagePath = parseArg('storage', path.resolve(process.cwd(), 'playwright-storage.json'));
  if (userDataDir) {
    // launch persistent context using a real profile dir to reduce automation fingerprints
    persistentContext = await chromium.launchPersistentContext(userDataDir, { ...launchOpts, ignoreDefaultArgs: ['--enable-automation'] });
    context = persistentContext;
  } else {
    browser = await chromium.launch(launchOpts);
  }
  // stealth mode: set common Chrome UA + languages + viewport to reduce automation fingerprint
    if (stealth) {
    const userAgent = parseArg('userAgent', 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36');
    // extra headers to mimic a real browser session
    const extraHeaders = {
      'accept-language': 'en-US,en;q=0.9',
      'sec-ch-ua': '"Chromium";v="141", "Not?A_Brand";v="8"',
      'sec-ch-ua-mobile': '?0',
      'sec-ch-ua-platform': '"Linux"'
    };
    const contextOpts = { userAgent, locale: 'en-US', viewport: { width: 1280, height: 800 }, extraHTTPHeaders: extraHeaders };
    if (persistentContext) {
      context = persistentContext;
    } else if (fs.existsSync(storagePath)) {
      console.log('Loading storage state from', storagePath);
      context = await browser.newContext({ ...contextOpts, storageState: storagePath });
    } else {
      context = await browser.newContext(contextOpts);
    }
  } else {
    if (persistentContext) {
      context = persistentContext;
    } else if (fs.existsSync(storagePath)) {
      console.log('Loading storage state from', storagePath);
      context = await browser.newContext({ storageState: storagePath });
    } else {
      context = await browser.newContext();
    }
  }
  const page = await context.newPage();
  let tokenHandled = false;

  // If stealth mode is requested, inject scripts to reduce automation fingerprints
  if (stealth) {



    await page.addInitScript(() => {
      // Stronger stealth: hide webdriver and provide realistic navigator properties
      try { Object.defineProperty(navigator, 'webdriver', { get: () => undefined, configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'], configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'plugins', { get: () => ({ length: 3 }), configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'mimeTypes', { get: () => ({ length: 3 }), configurable: true }); } catch (e) {}
      try { window.chrome = window.chrome || { runtime: {} }; } catch (e) {}
      try { Object.defineProperty(navigator, 'hardwareConcurrency', { get: () => 4, configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'deviceMemory', { get: () => 8, configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'maxTouchPoints', { get: () => 0, configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'platform', { get: () => 'Linux x86_64', configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'product', { get: () => 'Gecko', configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'vendor', { get: () => 'Google Inc.', configurable: true }); } catch (e) {}
      try { Object.defineProperty(navigator, 'connection', { get: () => ({ effectiveType: '4g', downlink: 10, rtt: 50 }), configurable: true }); } catch (e) {}

      // fake WebGL debug info
      try {
        const getParameter = WebGLRenderingContext.prototype.getParameter;
        WebGLRenderingContext.prototype.getParameter = function(param) {
          // 37445 = UNMASKED_VENDOR_WEBGL, 37446 = UNMASKED_RENDERER_WEBGL
          if (param === 37445) return 'Google Inc.';
          if (param === 37446) return 'ANGLE (Intel, Intel(R) UHD Graphics 620, OpenGL 4.1)';
          return getParameter.call(this, param);
        };
      } catch (e) {}

      // permissions: ensure query returns expected shape
      try {
        const origQuery = navigator.permissions.query;
        navigator.permissions.query = parameters => {
          if (parameters && parameters.name === 'notifications') {
            return Promise.resolve({ state: Notification.permission });
          }
          return origQuery(parameters);
        };
      } catch (e) {}

      // Overwrite toString to avoid detection via toString checks
      try {
        const _toString = Function.prototype.toString;
        const patchedToString = function() { return 'function toString() { [native code] }'; };
        Function.prototype.toString = new Proxy(Function.prototype.toString, {
          apply: function(target, thisArg, args) { return _toString.apply(thisArg, args); }
        });
      } catch (e) {}
    });
    console.log('Stealth mode enabled: injected anti-detection scripts and set common UA/locale/viewport');
  }

  const captures = [];
  const resourceTypesToSkip = new Set(['image', 'stylesheet', 'font', 'media', 'manifest', 'wasm']);

  // file suffixes and content-types to skip body capture for
  const suffixesToSkip = ['.js', '.png', '.html', '.css', '.wasm', '.otf', '.ttf', '.woff2'];
  const contentTypesToSkip = [
    'application/javascript', 'text/javascript', 'image/png', 'text/html', 'text/css', 'application/wasm',
    'font/otf', 'font/ttf', 'font/woff2', 'font/woff', 'application/font-woff', 'application/octet-stream'
  ];

  function isResourceRequest(reqOrResp) {
    try {
      // request.resourceType() exists on Request, response.request().resourceType() for Response
      const rt = typeof reqOrResp.resourceType === 'function' ? reqOrResp.resourceType() : (reqOrResp.request && typeof reqOrResp.request === 'function' ? null : null);
      return false; // fallback - we'll attempt other checks below
    } catch (e) {
      // ignore
    }
    return false;
  }

  page.on('request', request => {
    const resourceType = (request.resourceType && typeof request.resourceType === 'function') ? request.resourceType() : null;
    const url = request.url && request.url();
    const lowerUrl = url && url.toLowerCase();
    const hasSuffix = lowerUrl && suffixesToSkip.some(s => lowerUrl.endsWith(s));
    const isResource = (resourceType && resourceTypesToSkip.has(resourceType)) || hasSuffix;
    // For resource requests, only store minimal metadata and avoid postData
    const entry = {
      type: 'request',
      url: request.url(),
      method: request.method(),
      headers: request.headers(),
      resourceType: resourceType || 'other',
      timestamp: Date.now()
    };
    if (!isResource) {
      // include post data for non-resources
      entry.postData = request.postData();
    }
    captures.push(entry);
    console.log('REQ', request.method(), request.url(), isResource ? `[skipped-body:${resourceType}]` : '');
  });

  page.on('response', async response => {
    const req = response.request && response.request();
    const resourceType = req && req.resourceType && typeof req.resourceType === 'function' ? req.resourceType() : null;
    const url = response.url && response.url();
    const lowerUrl = url && url.toLowerCase();
    const hasSuffix = lowerUrl && suffixesToSkip.some(s => lowerUrl.endsWith(s));
    const headers = response.headers && typeof response.headers === 'function' ? response.headers() : response.headers || {};
    const contentType = headers && Object.keys(headers).reduce((acc, k) => {
      if (k.toLowerCase() === 'content-type') return headers[k];
      return acc;
    }, null);
    const hasContentType = contentType && contentTypesToSkip.some(ct => contentType.toLowerCase().includes(ct));
    const isResource = (resourceType && resourceTypesToSkip.has(resourceType)) || hasSuffix || hasContentType;
    let body = null;
    if (!isResource) {
      try {
        body = await response.text();
      } catch (e) {
        body = `<<unable to read: ${e.message}>>`;
      }
    }
    const entry = {
      type: 'response',
      url: response.url(),
      status: response.status(),
      headers: response.headers(),
      resourceType: resourceType || 'other',
      timestamp: Date.now()
    };
    if (!isResource) entry.body = body;
    captures.push(entry);
    console.log('RES', response.status(), response.url(), isResource ? `[skipped-body:${resourceType}]` : '');
    // If the response is from the Matrix login endpoint and contains an M_BAD_JSON error, persist and exit
    try {
      const urlStr = response.url();
      if (!isResource && urlStr.includes('/_matrix/client/v3/login') && body) {
        let parsed = null;
        try {
          parsed = JSON.parse(body);
        } catch (e) {
          // not JSON - ignore
        }
        if (parsed && parsed.errcode === 'M_BAD_JSON') {
          console.error('Received M_BAD_JSON from login endpoint; saving captures and exiting');
          try {
            const out = {
              meta: { capturedAt: new Date().toISOString(), target: homeserver ? 'homeserver-sso-redirect' : 'https://fluffychat.im/web/#/home', homeserver, redirectUrl, headless },
              captures
            };
            fs.writeFileSync('captures.json', JSON.stringify(out, null, 2));
            console.log('Saved captures.json');
          } catch (e) {
            console.error('Failed to write captures.json:', e && e.message);
          }
          try {
            if (browser) await browser.close();
            if (persistentContext) await persistentContext.close();
          } catch (e) {
            // ignore
          }
          process.exit(3);
        }
      }
    } catch (e) {
      // swallow to avoid crashing the capture loop
      console.warn('Error while checking for M_BAD_JSON:', e && e.message);
    }
    // If a redirect back to an SPA contains a token in fragment, save it and trigger immediate login
    try {
      const u = response.url();
      if (!tokenHandled && u.includes('fluffychat.im') && u.includes('token=')) {
        const path = require('path');
        const fs = require('fs');
        const parsed = new URL(u);
        const frag = parsed.hash || '';
        // hash may be like '#/home?token=...'
        if (frag.includes('token=')) {
          tokenHandled = true;
          const now = new Date().toISOString();
          const out = { capturedAt: now, url: u };
          fs.writeFileSync(path.resolve(process.cwd(), 'last_token.json'), JSON.stringify(out, null, 2));
          console.log('Saved last_token.json for immediate login trigger');

          // Perform the POST from the page (browser context) to preserve same-origin/credentials behavior
          try {
            const homeserverArg = homeserver || 'https://test.aaca.eu.org';
            (async () => {
              try {
                const fetchResult = await page.evaluate(async (callbackUrl, hs) => {
                  // extract token from fragment like '#/home?token=...&user_id=...'
                  try {
                    const u = new URL(callbackUrl);
                    const frag = u.hash || '';
                    let qs = '';
                    if (frag.includes('?')) qs = frag.split('?',1)[1];
                    else qs = frag.replace(/^#/, '');
                    const params = new URLSearchParams(qs);
                    const token = params.get('token');
                    const body = { type: 'm.login.token', token, initial_device_display_name: 'Playwright-browser', refresh_token: false };
                    const resp = await fetch(hs.replace(/\/+$/,'') + '/_matrix/client/v3/login', {
                      method: 'POST',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify(body),
                      credentials: 'include',
                      mode: 'cors'
                    });
                    const text = await resp.text();
                    return { status: resp.status, body: text };
                  } catch (e) {
                    return { error: String(e) };
                  }
                }, u, homeserverArg);
                console.log('Browser-context login fetch result:', JSON.stringify(fetchResult));
              } catch (err) {
                console.error('Error during browser-context fetch:', err && err.message);
              }
            })();
          } catch (e) {
            console.error('Failed to invoke browser fetch for token login:', e && e.message);
          }
        }
      }
    } catch (e) {
      console.warn('token detection hook failed:', e && e.message);
    }
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

  // Always start by opening FluffyChat. The user will perform the interactive steps in the opened browser.
  console.log('Opening https://fluffychat.im/web/#/home');
  try {
    await page.goto('https://fluffychat.im/web/#/home', { waitUntil: 'networkidle' });
  } catch (e) {
    console.log('Navigation error while opening FluffyChat (may be normal):', e.message);
  }

  if (prefillHome) {
      // Wait for the hidden form/input used by FluffyChat and set the homeserver value
      try {
        await page.waitForSelector('form.transparentTextEditing input.flt-text-editing', { timeout: 15000 });
        await page.evaluate((server) => {
          const input = document.querySelector('form.transparentTextEditing input.flt-text-editing');
          if (input) {
            input.focus();
            input.value = server;
            // dispatch input events so any listeners update internal state
            input.dispatchEvent(new Event('input', { bubbles: true }));
            input.dispatchEvent(new Event('change', { bubbles: true }));
          }
        }, homeserver || 'test.aaca.eu.org');
        console.log('Prefilled homeserver in FluffyChat input. Please click the login button in the opened browser to start SSO.');
      } catch (e) {
        console.log('Prefill failed or selector not found:', e.message);
      }

      // Wait for navigation to the provider authorize page or for manual Enter from user
      try {
        // Wait up to 2 minutes for provider authorize URL to appear
        await page.waitForURL('**/oauth2/authorize**', { timeout: 120000 });
        console.log('Detected navigation to provider authorize page');
      } catch (e) {
        console.log('Provider authorize page not detected automatically; you can continue manually.');
      }

      // If we are now on the provider authorize page, wait for the approve button and click it
      try {
        // Approve buttons are links like /oauth2/approve/..., wait for one and click
        await page.waitForSelector('a[href^="/oauth2/approve"]', { timeout: 30000 });
        await page.click('a[href^="/oauth2/approve"]');
        console.log('Clicked approve button on provider page');
      } catch (e) {
        console.log('Approve button not found or click failed:', e.message);
      }
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

  try {
    if (browser) await browser.close();
    if (persistentContext) await persistentContext.close();
  } catch (e) {
    // ignore close errors
  }
}

main().catch(err => {
  console.error('Script error:', err);
  process.exit(2);
});
