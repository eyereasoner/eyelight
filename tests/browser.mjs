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
  for (let i = 0; i < 200; i++) { if (await evaluate(expression)) return; await delay(50); }
  throw new Error(`Timed out: ${label}`);
}
async function navigate(url) {
  await command('Page.navigate', { url });
  await wait('document.readyState === "complete" && !!document.getElementById("run") && !document.getElementById("run").disabled && !document.getElementById("source").disabled', 'page ready');
  await delay(80);
}
async function set(id, value, event = 'input') {
  await evaluate(`document.getElementById(${JSON.stringify(id)}).value = ${JSON.stringify(value)}; document.getElementById(${JSON.stringify(id)}).dispatchEvent(new Event(${JSON.stringify(event)}, { bubbles: true }));`);
}
async function run(mode) {
  if (mode) await set('mode', mode, 'change');
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
  assert.match(await run('answers'), /Who = socrates/);
  assert.match(await run('explain'), /rule 1; uses \[1\]/);
  assert.equal(await evaluate('document.getElementById("version").textContent'), `v${expectedVersion}`);
  assert.equal(await evaluate('document.querySelector(".editors .panel-heading > span").textContent'), '.pl');
  const widths = await evaluate('[...document.querySelectorAll(".editors > .panel")].map(e => e.getBoundingClientRect().width)');
  assert.ok(Math.abs(widths[0] - widths[1]) < 1);
  assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth'));
  await screenshot('desktop');

  const generated = await run('checked');
  assert.match(generated, /checked: 2 steps, 1 claim/);
  for (let id = 1; id <= 9; id++) assert.match(generated, new RegExp(`C${id} `));
  await screenshot('desktop-proof-check');

  const proof = await run('proof');
  assert.match(proof, /^eyelight-proof 1/);
  await set('saved-proof', proof);
  const checked = await run('check');
  assert.match(checked, /checked: 2 steps, 1 claim/);
  for (let id = 1; id <= 9; id++) assert.match(checked, new RegExp(`C${id} `));
  assert.match(checked, /C5 re-decision: not applicable/);
  assert.match(checked, /C6 boundary consistency: not applicable/);
  assert.match(checked, /C8 witness integrity: not applicable/);
  assert.equal(await evaluate('document.getElementById("saved-proof-panel").hidden'), false);
  await set('saved-proof', proof.replace('mortal(socrates)', 'mortal(plato)'));
  const invalid = await run('check');
  assert.match(invalid, /invalid: 2 steps/);
  assert.match(invalid, /\[C1 resolution\] step 2/);
  await set('source', 'human(X).');
  assert.match(await run('answers'), /fact must be ground/);
  assert.ok(await evaluate('document.getElementById("output").classList.contains("error")'));

  for (const name of ['arithmetic', 'negation', 'aggregation', 'witnesses', 'orders']) {
    await set('example', name, 'change');
    await wait(`!document.getElementById("source").disabled && document.getElementById("example").value === ${JSON.stringify(name)}`, `${name} loaded`);
    const result = await run('checked');
    assert.match(result, /^checked:/, name);
    if (name === 'orders') {
      assert.doesNotMatch(result, /not applicable/);
      for (let i = 1; i <= 9; i++) assert.match(result, new RegExp(`C${i} [^\n]+: [1-9][0-9]* checked, 0 failed`));
      const certificate = await run('proof');
      const forged = certificate.replace('ge(15, 10)', 'ge(0, 10)');
      assert.notEqual(forged, certificate);
      await set('saved-proof', forged);
      assert.match(await run('check'), /\[C5 re-decision\]/);
    }
  }

  await set('example', 'deep-taxonomy-100000', 'change');
  await wait('!document.getElementById("source").disabled && document.getElementById("example").value === "deep-taxonomy-100000"', 'deep taxonomy loaded');
  assert.equal(await run('answers'), '?- test(Result).\nResult = true.\n');
  const deepReport = await run('checked');
  assert.match(deepReport, /^checked: 100003 steps, 1 claim/);
  await set('mode', 'answers', 'change');
  await evaluate('document.getElementById("share").click()');
  await wait('!document.getElementById("share-box").hidden', 'deep taxonomy share opened');
  assert.equal(new URL(await evaluate('document.getElementById("share-url").value')).hash, '#example=deep-taxonomy-100000');

  await navigate(`${base}#example=path-discovery`);
  const airportAnswers = await run('answers');
  assert.equal((airportAnswers.match(/Stops = 2/g) || []).length, 3);
  for (const name of ['Heraklion International Nikos Kazantzakis Airport', 'Diagoras Airport', 'Palma De Mallorca Airport']) {
    assert.ok(airportAnswers.includes(`Liège Airport -> ${name} -> Václav Havel Airport Prague`));
  }
  assert.match(await run('checked'), /^checked: [0-9]+ steps, 3 claims/);
  await set('mode', 'answers', 'change');
  await evaluate('document.getElementById("share").click()');
  await wait('!document.getElementById("share-box").hidden', 'airport share opened');
  assert.equal(new URL(await evaluate('document.getElementById("share-url").value')).hash, '#example=path-discovery');
  await screenshot('desktop-airport-routes');

  await set('example', 'paths', 'change');
  await wait('!document.getElementById("source").disabled && document.getElementById("source").value.includes("museum")', 'paths loaded');
  const paths = await run('answers');
  for (const name of ['park', 'station', 'museum']) assert.match(paths, new RegExp(`Where = ${name}`));
  await evaluate('document.getElementById("share").click()');
  await wait('!document.getElementById("share-box").hidden', 'share opened');
  assert.equal(new URL(await evaluate('document.getElementById("share-url").value')).hash, '#example=paths');

  const unicode = "human('Élodie'). mortal(P) :- human(P). ?- mortal(Who).";
  await set('source', unicode);
  await set('saved-proof', proof);
  await set('mode', 'explain', 'change');
  await evaluate('document.getElementById("share").click()');
  const shared = await evaluate('document.getElementById("share-url").value');
  const params = new URLSearchParams(new URL(shared).hash.slice(1));
  assert.equal(params.get('code'), unicode);
  assert.equal(params.get('proof'), proof);
  assert.equal(params.get('mode'), 'explain');
  await navigate(shared);
  assert.equal(await evaluate('document.getElementById("source").value'), unicode);
  assert.equal(await evaluate('document.getElementById("saved-proof").value'), proof);
  assert.match(await evaluate('document.getElementById("output").textContent'), /first answer/);
  assert.match(await run(), /Élodie/);

  // Cancel before worker initialization resolves, then run successfully again.
  await navigate(`${base}#example=socrates`);
  await evaluate('document.getElementById("run").click(); document.getElementById("stop").click()');
  assert.equal(await evaluate('document.getElementById("status").textContent'), 'Stopped.');
  assert.match(await run('answers'), /socrates/);
  // A larger closure keeps the worker busy while the UI remains cancellable.
  const chain = Array.from({ length: 250 }, (_, i) => `edge(n${i},n${i + 1}).`).join('\n')
    + '\npath(X,Y) :- edge(X,Y). path(X,Z) :- path(X,Y), edge(Y,Z). ?- path(n0,Y).';
  await set('source', chain);
  await evaluate('document.getElementById("run").click()');
  await delay(20);
  await evaluate('document.getElementById("stop").click()');
  assert.equal(await evaluate('document.getElementById("status").textContent'), 'Stopped.');
  await set('source', unicode);
  assert.match(await run('answers'), /Élodie/);
  await delay(250);
  await navigate(base);
  assert.equal(await evaluate('document.getElementById("source").value'), unicode);
  assert.match(await evaluate('document.getElementById("output").textContent'), /first answer/);

  await command('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: true });
  await navigate(`${base}#example=socrates`);
  const phoneReport = await run('checked');
  assert.match(phoneReport, /checked: 2 steps, 1 claim/);
  for (let id = 1; id <= 9; id++) assert.match(phoneReport, new RegExp(`C${id} `));
  assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth'), 'phone layout overflows');
  assert.ok(await evaluate('document.querySelector(".output-panel").getBoundingClientRect().top > document.querySelector(".editors .panel").getBoundingClientRect().top'), 'phone panels should stack');
  await screenshot('mobile');
  await set('example', 'orders', 'change');
  await wait('!document.getElementById("source").disabled && document.getElementById("example").value === "orders"', 'mobile voucher loaded');
  const mobileVoucher = await run('checked');
  assert.match(mobileVoucher, /^checked:/);
  assert.doesNotMatch(mobileVoucher, /not applicable/);
  assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth'), 'mobile voucher layout overflows');
  await screenshot('mobile-all-conditions');
  await set('example', 'deep-taxonomy-1000', 'change');
  await wait('!document.getElementById("source").disabled && document.getElementById("example").value === "deep-taxonomy-1000"', 'mobile taxonomy loaded');
  assert.match(await run('checked'), /^checked: 1003 steps, 1 claim/);
  assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth'), 'mobile taxonomy layout overflows');

  await set('example', 'path-discovery', 'change');
  await wait('!document.getElementById("source").disabled && document.getElementById("example").value === "path-discovery"', 'mobile airport graph loaded');
  assert.match(await run('checked'), /^checked: [0-9]+ steps, 3 claims/);
  assert.ok(await evaluate('document.documentElement.scrollWidth <= innerWidth'), 'mobile airport layout overflows');
  await screenshot('mobile-airport-routes');

  if (pagesRoot) {
    // Check the compatibility route as well as the directory route used above.
    await navigate(new URL('playground.html#example=socrates', pagesRoot).href);
    assert.match(await run('checked'), /checked: 2 steps, 1 claim/);
    for (const [name, width, mobile] of [['desktop', 1280, false], ['mobile', 390, true]]) {
      await command('Emulation.setDeviceMetricsOverride', { width, height: 1000, deviceScaleFactor: 1, mobile });
      for (const route of ['', 'SPEC/']) {
        const url = new URL(route, pagesRoot).href;
        await command('Page.navigate', { url });
        await wait(`location.href === ${JSON.stringify(url)} && document.readyState === "complete" && !!document.querySelector(".documentation article h1")`, 'rendered documentation');
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
  console.log('Browser checks passed: full airport path discovery and sharing on desktop/mobile, depth-100000 taxonomy, short taxonomy share, mobile taxonomy, all nine proof conditions on desktop/mobile, new examples, forged built-ins, real Wasm, answers, explanations, proof round trip, forged proof, errors, short/snapshot shares, Unicode, autosave, no autorun, cold/hot cancellation, restart, equal desktop columns, and phone layout.');
} finally { socket.close(); }
