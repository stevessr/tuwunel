Playwright network capture for FluffyChat

Instructions (fish shell):

1. Install dependencies

   npm install

2. Run the capture script

   npm run capture

This will create a `captures.json` file in the repository root containing requests and responses observed while loading https://fluffychat.im/web/#/home

Storage state and interactive headful run
----------------------------------------

To persist cookies/localStorage across runs (so you can login once interactively and reuse the session later), the script will read/write a storage state file named `playwright-storage.json` by default.

Example workflows:

- Run headful and complete Cloudflare Turnstile manually, then press Enter in the terminal to let the script save state and capture network requests:

   node tests/capture-login.js --homeserver=https://test.aaca.eu.org --redirectUrl=https://fluffychat.im/web/%23/home --headless=false

- Use a custom storage file path:

   node tests/capture-login.js --storage=/tmp/my-storage.json --homeserver=https://test.aaca.eu.org --headless=false

After manual login and pressing Enter, the storage state will be saved and subsequent runs (including headless) will load that state so SSO/Turnstile won't need to be repeated.

Stealth (reduce automation fingerprint)
-------------------------------------

If Cloudflare or other bot protections detect headless/automation, you can enable a simple "stealth" mode which sets a common Chrome user-agent, locale, viewport and injects small scripts to reduce obvious automation signals. It's not a guarantee but helps in many cases.

Example:

   node tests/capture-login.js --stealth=true --headless=false --homeserver=https://test.aaca.eu.org

If your redirect URL contains a fragment (#) and you want to pass it unencoded to the server, use `--noEncodeRedirect=true` and quote the argument in fish shell, for example:

   node tests/capture-login.js --stealth=true --headless=false --homeserver=https://test.aaca.eu.org --redirectUrl 'https://fluffychat.im/web/#/home' --noEncodeRedirect=true


