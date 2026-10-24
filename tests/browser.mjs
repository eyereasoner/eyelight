// Optional real-browser check. Start the local HTTP server and Chrome with
// --remote-debugging-port=9226, then run this script with Node.js 22 or later.
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';

const base = process.argv[2] || 'http://127.0.0.1:8766/playground.html';
const debug = process.argv[3] || 'http://127.0.0.1:9226';
const screenshots = process.argv[4];
const pagesRoot = process.argv[5];
const expectedVersion = /^version\s*=\s*"([^"]+)"/m.exec(await readFile(new URL('../Cargo.toml', import.meta.url), 'utf8'))[1];
const tabs = await (await fetch(`${debug}/json`)).json();
const tab = tabs.find(tab => tab.type === 'page');
assert.ok(tab, 'Chrome needs an open page');
const socket = new WebSocket(tab.webSocketDebuggerUrl);
await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
let nextId = 0;
const pending = new Map();
const exceptions = [];
socket.onmessage = ({ data }) => {
  const message = JSON.parse(data);
  if (message.id) {
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) reject(new Error(JSON.stringify(message.error)));
    else resolve(message.result);
  } else if (message.method === 'Runtime.exceptionThrown') exceptions.push(message.params);
};
function command(method, params = {}) {
  const id = ++nextId;
  return new Promise((resolve, reject) => { pending.set(id, { resolve, reject }); socket.send(JSON.stringify({ id, method, params })); });
}
async function evaluate(expression) {
  const response = await command('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  assert.ok(!response.exceptionDetails, JSON.stringify(response.exceptionDetails));
  return response.result.value;
}
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
async function wait(expression, label) {
  for (let i = 0; i < 1200; i++) { if (await evaluate(expression)) return; await delay(50); }
  throw new Error(`Timed out: ${label}`);
}
async function navigate(url) {
  await command('Page.navigate', { url });
  await wait(`location.href === ${JSON.stringify(url)} && document.readyState === "complete" && !!document.getElementById("run") && !document.getElementById("run").disabled && !document.getElementById("source").disabled`, 'page ready');
  const params = new URLSearchParams(new URL(url).hash.slice(1));
  if (params.has('example')) {
    await wait(`document.getElementById("example").value === ${JSON.stringify(params.get('example'))} && !document.getElementById("source").disabled`, 'shared example loaded');
  } else if (params.has('code')) {
    await wait(`document.getElementById("source").value === ${JSON.stringify(params.get('code'))}`, 'shared source restored');
  }
  await delay(80);
}
async function set(id, value, event = 'input') {
  await evaluate(`document.getElementById(${JSON.stringify(id)}).value = ${JSON.stringify(value)}; document.getElementById(${JSON.stringify(id)}).dispatchEvent(new Event(${JSON.stringify(event)}, { bubbles: true }));`);
}
async function run() {
  await evaluate('document.getElementById("run").click()');
  await wait('!document.getElementById("run").disabled', 'run completion');
  return evaluate('document.getElementById("output").textContent');
}
async function screenshot(name) {
  if (!screenshots) return;
  await mkdir(screenshots, { recursive: true });
  const metrics = await command('Page.getLayoutMetrics');
  const { width, height } = metrics.cssContentSize;
  const image = await command('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true, clip: { x: 0, y: 0, width, height, scale: 1 } });
  await writeFile(`${screenshots}/${name}.png`, Buffer.from(image.data, 'base64'));
}

try {
  await command('Page.enable');
  await command('Runtime.enable');
  await command('Page.navigate', { url: 'about:blank' });
  await wait('location.href === "about:blank" && document.readyState === "complete"', 'fresh page');
  await command('Emulation.setDeviceMetricsOverride', { width: 1280, height: 1000, deviceScaleFactor: 1, mobile: false });
  await navigate(`${base}#example=socrates`);
  assert.match(await evaluate('document.getElementById("output").textContent'), /first answer/);
  assert.equal(await evaluate('document.querySelector("#mode, #saved-proof")'), null);
  assert.equal(await evaluate('document.getElementById("version").textContent'), `v${expectedVersion}`);
  const socrates = await run();
  assert.match(socrates, /\?- mortal\(Who\)\.\nWho = socrates/);
  assert.match(socrates, /\?- why\.\nwheye-proof 1/);
  assert.match(socrates, /\?- check\.\nchecked: 2 steps, 1 claim/);
  for (let id = 1; id <= 9; id++) assert.match(socrates, new RegExp(`C${id} `));
  const widths = await evaluate('[...document.querySelectorAll(".editors > .panel")].map(e => e.getBoundingClientRect().width)');
  assert.ok(Math.abs(widths[0] - widths[1]) < 1);
  assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth'));
  await screenshot('desktop');

  await set('source', 'p(a). ?- p(X). ?- check(p(b)). ?- why. ?- check.');
  const explicit = await run();
  assert.match(explicit, /^\?- p\(X\)\.\nX = a\.\n\n\?- check\(p\(b\)\)\.\nfalse\./);
  assert.match(explicit, /checked: 1 step, 1 claim/);
  await set('source', '?- check(true). ?- check(false).');
  assert.equal(await run(), '?- check(true).\ntrue.\n\n?- check(false).\nfalse.\n');
  await set('source', 'human(X). ?- human(X).');
  assert.match(await run(), /ground answer/);
  assert.ok(await evaluate('document.getElementById("output").classList.contains("error")'));

  async function example(name) {
    await set('example', name, 'change');
    await wait(`!document.getElementById("source").disabled && document.getElementById("example").value === ${JSON.stringify(name)}`, `${name} loaded`);
    return evaluate('document.getElementById("source").value');
  }
  for (const name of ['arithmetic', 'negation', 'aggregation', 'witnesses', 'orders']) {
    const source = await example(name);
    await set('source', source + '\n?- check.');
    const result = await run();
    assert.match(result, /^checked:/m, name);
    if (name === 'orders') {
      assert.doesNotMatch(result, /not applicable/);
      for (let i = 1; i <= 9; i++) assert.match(result, new RegExp(`C${i} [^\n]+: [1-9][0-9]* checked, 0 failed`));
    }
  }

  const deep = await example('deep-taxonomy-100000');
  assert.equal(await run(), '?- test(Result).\nResult = true.\n');
  await set('source', deep + '\n?- check.');
  assert.match(await run(), /^checked: 100003 steps, 1 claim/m);
  await set('source', deep);
  await evaluate('document.getElementById("share").click()');
  await wait('!document.getElementById("share-box").hidden', 'deep taxonomy share opened');
  assert.equal(new URL(await evaluate('document.getElementById("share-url").value')).hash, '#example=deep-taxonomy-100000');

  await navigate(`${base}#example=path-discovery`);
  const airport = await evaluate('document.getElementById("source").value');
  await set('source', airport + '\n?- check.');
  const routes = await run();
  assert.equal((routes.match(/Stopovers = 2/g) || []).length, 3, routes.slice(0, 1000));
  for (const name of ['Heraklion International Nikos Kazantzakis Airport', 'Diagoras Airport', 'Palma De Mallorca Airport']) {
    assert.ok(routes.includes(`Liège Airport -> ${name} -> Václav Havel Airport Prague`));
  }
  assert.match(routes, /^checked: [0-9]+ steps, 3 claims/m);
  await set('source', airport);
  await evaluate('document.getElementById("share").click()');
  assert.equal(new URL(await evaluate('document.getElementById("share-url").value')).hash, '#example=path-discovery');
  await screenshot('desktop-airport-routes');
  await example('paths');
  for (const name of ['ben', 'cara']) assert.match(await run(), new RegExp(`Who = ${name}`));
  await evaluate('document.getElementById("share").click()');
  assert.equal(new URL(await evaluate('document.getElementById("share-url").value')).hash, '#example=paths');

  const unicode = "human('Élodie'). mortal(P) :- human(P). ?- mortal(Who). ?- why. ?- check.";
  await set('source', unicode);
  await evaluate('document.getElementById("share").click()');
  const shared = await evaluate('document.getElementById("share-url").value');
  const params = new URLSearchParams(new URL(shared).hash.slice(1));
  assert.equal(params.get('code'), unicode);
  assert.deepEqual([...params.keys()], ['code']);
  await navigate(shared);
  assert.equal(await evaluate('document.getElementById("source").value'), unicode);
  assert.match(await evaluate('document.getElementById("output").textContent'), /first answer/);
  assert.match(await run(), /Élodie/);

  await navigate(`${base}#example=socrates`);
  await evaluate('document.getElementById("run").click(); document.getElementById("stop").click()');
  assert.equal(await evaluate('document.getElementById("status").textContent'), 'Stopped.');
  assert.match(await run(), /socrates/);
  const chain = Array.from({ length: 250 }, (_, i) => `edge(n${i},n${i + 1}).`).join('\n')
    + '\npath(X,Y) :- edge(X,Y). path(X,Z) :- path(X,Y), edge(Y,Z). ?- path(n0,Y).';
  await set('source', chain);
  await evaluate('document.getElementById("run").click()');
  await delay(20);
  await evaluate('document.getElementById("stop").click()');
  assert.equal(await evaluate('document.getElementById("status").textContent'), 'Stopped.');
  await set('source', unicode);
  assert.match(await run(), /Élodie/);
  await delay(250);
  await navigate(base);
  assert.equal(await evaluate('document.getElementById("source").value'), unicode);
  assert.match(await evaluate('document.getElementById("output").textContent'), /first answer/);

  await command('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: true });
  await navigate(`${base}#example=socrates`);
  assert.match(await run(), /checked: 2 steps, 1 claim/);
  assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth'), 'phone layout overflows');
  assert.ok(await evaluate('document.querySelector(".output-panel").getBoundingClientRect().top > document.querySelector(".editors .panel").getBoundingClientRect().top'), 'phone panels should stack');
  await screenshot('mobile');
  for (const name of ['orders', 'deep-taxonomy-1000', 'path-discovery']) {
    const source = await example(name);
    await set('source', source + '\n?- check.');
    const result = await run();
    assert.match(result, /^checked:/m);
    if (name === 'orders') assert.doesNotMatch(result, /not applicable/);
    if (name === 'deep-taxonomy-1000') assert.match(result, /checked: 1002 steps, 1 claim/);
    if (name === 'path-discovery') assert.match(result, /checked: [0-9]+ steps, 3 claims/);
    assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth'), 'mobile layout overflows');
    await screenshot(`mobile-${name}`);
  }

  if (pagesRoot) {
    // Check the compatibility route as well as the directory route used above.
    await navigate(new URL('playground.html#example=socrates', pagesRoot).href);
    assert.match(await run(), /checked: 2 steps, 1 claim/);
    for (const [name, width, mobile] of [['desktop', 1280, false], ['mobile', 390, true]]) {
      await command('Emulation.setDeviceMetricsOverride', { width, height: 1000, deviceScaleFactor: 1, mobile });
      for (const route of ['', 'SPEC/']) {
        const url = new URL(route, pagesRoot).href;
        await command('Page.navigate', { url });
        await wait(`location.href === ${JSON.stringify(url)} && document.readyState === "complete" && !!document.querySelector(".documentation h1")`, 'rendered documentation');
        assert.equal(await evaluate('document.querySelectorAll(".documentation h1").length'), 1, 'Documentation needs one page title');
        assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth'), `${name} documentation overflows`);
        assert.ok(await evaluate('[...document.images].every(image => image.complete && image.naturalWidth > 0)'));
        const links = await evaluate('[...document.querySelectorAll("a[href]")].map(a => a.href)');
        assert.ok(links.every(link => !/\.md(?:#|$)/.test(link)), 'documentation links should open rendered pages');
        for (const link of new Set(links)) {
          const target = new URL(link);
          if (target.origin !== new URL(pagesRoot).origin) continue;
          assert.ok(target.pathname.startsWith(new URL(pagesRoot).pathname), 'link escaped the Pages base path');
          assert.ok(await evaluate(`fetch(${JSON.stringify(link)}).then(response => response.ok)`), `broken link: ${link}`);
        }
        if (route) assert.ok(await evaluate('!!document.getElementById("8-verification")'), 'proof contract anchor missing');
        await screenshot(`${route ? 'specification' : 'guide'}-${name}`);
      }
    }
  }
  assert.deepEqual(exceptions, []);
  console.log('Browser checks passed: source-selected answers, proofs and C1–C9 checks, echoed queries, all airport routes, depth-100000 taxonomy, desktop/mobile layouts, rendered docs, sharing, Unicode, autosave, cancellation and restart.');
} finally { socket.close(); }
