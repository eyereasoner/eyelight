import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import vm from 'node:vm';
import { createHash, webcrypto } from 'node:crypto';

const root = new URL('../', import.meta.url);
test('all examples are selectable and available to shared links', async () => {
  const html = await readFile(new URL('playground.html', root), 'utf8');
  const js = await readFile(new URL('playground.js', root), 'utf8');
  for (const file of await readdir(new URL('examples/', root))) {
    if (!file.endsWith('.pl')) continue;
    const name = file.slice(0, -3);
    assert.ok(js.includes(`'${name}'`), `${name} missing from share allowlist`);
    assert.ok(html.includes(`value="${name}"`));
    assert.match(await readFile(new URL(`examples/${name}.pl`, root), 'utf8'), /\?- /);
  }
});
test('worker runs the engine off the UI thread and reports failures', async () => {
  const source = await readFile(new URL('playground-worker.js', root), 'utf8');
  const messages = [];
  const context = vm.createContext({ self: { postMessage: message => messages.push(message) } });
  const script = new vm.Script(source.replace("import('./pkg/wheye.js')", 'Promise.resolve(mockEngine)'), { filename: fileURLToPath(new URL('playground-worker.js', root)) });
  context.mockEngine = { default: async () => {}, version: () => '0.1.0', runProgram: source => {
    if (source === 'bad') throw new Error('specific failure');
    return source;
  } };
  script.runInContext(context);
  await context.self.onmessage({ data: { kind: 'init' } });
  assert.equal(messages[0].version, '0.1.0');
  await context.self.onmessage({ data: { kind: 'run', id: 2, source: 'hello' } });
  assert.equal(messages[1].output, 'hello');
  await context.self.onmessage({ data: { kind: 'run', id: 3, source: 'bad' } });
  assert.equal(messages[2].kind, 'error');
  assert.match(messages[2].message, /specific failure/);
});

test('the source is the only input to execution and sharing', async () => {
  const html=await readFile(new URL('playground.html',root),'utf8');
  const js=await readFile(new URL('playground.js',root),'utf8');
  assert.doesNotMatch(html,/id="(?:mode|saved-proof)"/);
  assert.doesNotMatch(js,/get\('mode'\)|get\('proof'\)|set\('mode'\)|set\('proof'\)/);
});

test('the example index recognizes every current source and the previous Socrates source', async () => {
  const index = JSON.parse(await readFile(new URL('examples/index.json', root), 'utf8'));
  const names = new Set();
  for (const file of await readdir(new URL('examples/', root))) {
    if (!file.endsWith('.pl')) continue;
    const source = await readFile(new URL(`examples/${file}`, root), 'utf8');
    const hash = createHash('sha256').update(source.replace(/\r\n?/g, '\n')).digest('hex');
    assert.equal(index[hash], file.slice(0, -3), 'Run node scripts/index-examples.mjs after changing examples.');
    names.add(file.slice(0, -3));
  }
  for (const [hash, name] of Object.entries(index)) {
    assert.match(hash, /^[a-f0-9]{64}$/);
    assert.ok(names.has(name));
  }
  const legacy = await readFile(new URL('tests/fixtures/legacy-socrates.pl', root));
  assert.equal(index[createHash('sha256').update(legacy).digest('hex')], 'socrates');
});

async function playground({ source, hash = '' } = {}) {
  const elements = new Map();
  const timers = new Map();
  let nextTimer = 0;
  let saved = source === undefined ? null : JSON.stringify({ source });
  const location = new URL(`https://example.test/wheye/playground/${hash}`);
  const element = id => {
    if (!elements.has(id)) elements.set(id, {
      value: '', textContent: '', hidden: true, listeners: {},
      classList: { toggle() {} },
      addEventListener(event, listener) { this.listeners[event] = listener; },
    });
    return elements.get(id);
  };
  const context = vm.createContext({
    URL, URLSearchParams, TextEncoder, crypto: webcrypto, location,
    document: { getElementById: element },
    window: { addEventListener() {} },
    history: { replaceState(_state, _unused, url) { location.href = String(url); } },
    localStorage: { getItem: () => saved, setItem: (_key, value) => { saved = value; } },
    navigator: { clipboard: { writeText: async () => {} } },
    setTimeout: callback => { timers.set(++nextTimer, callback); return nextTimer; },
    clearTimeout: id => timers.delete(id),
    Worker: class { constructor() { throw new Error('Restoring a program must not run it.'); } },
    fetch: async url => {
      const source = await readFile(url, 'utf8');
      return { ok: true, text: async () => source, json: async () => JSON.parse(source) };
    },
  });
  const script = (await readFile(new URL('playground.js', root), 'utf8'))
    .replaceAll('import.meta.url', JSON.stringify(new URL('playground.js', root).href));
  await new vm.Script(`(async () => { ${script} })()`).runInContext(context);
  return {
    element, location,
    async share() {
      await element('share').listeners.click();
      return new URL(element('share-url').value);
    },
    autosave() {
      for (const callback of timers.values()) callback();
      timers.clear();
      return saved === null ? null : JSON.parse(saved).source;
    },
  };
}

test('fresh and restored unedited examples share short links', async () => {
  const fresh = await playground();
  assert.equal((await fresh.share()).hash, '#example=socrates');
  const source = fresh.element('source').value;
  const restored = await playground({ source });
  assert.equal(restored.element('example').value, 'socrates');
  assert.equal((await restored.share()).hash, '#example=socrates');
  const code = await playground({ hash: `#${new URLSearchParams({ code: source })}` });
  assert.equal(code.location.hash, '#example=socrates');
  assert.equal((await code.share()).hash, '#example=socrates');
});

test('old bundled examples restore current syntax from autosave and full source links', async () => {
  const legacy = await readFile(new URL('tests/fixtures/legacy-socrates.pl', root), 'utf8');
  const current = await readFile(new URL('examples/socrates.pl', root), 'utf8');
  for (const options of [{ source: legacy }, { hash: `#${new URLSearchParams({ code: legacy })}` }]) {
    const page = await playground(options);
    assert.equal(page.element('source').value, current);
    assert.equal(page.autosave(), options.source ? current : null);
    assert.equal((await page.share()).hash, '#example=socrates');
    assert.match(page.element('output').textContent, /first answer/);
    if (options.hash) assert.equal(page.location.hash, '#example=socrates');
  }
});

test('opening a shared example preserves an unrelated saved draft', async () => {
  const draft = "human('Élodie'). ?- human(Who).";
  const legacy = await readFile(new URL('tests/fixtures/legacy-socrates.pl', root), 'utf8');
  const page = await playground({ source: draft, hash: `#${new URLSearchParams({ code: legacy })}` });
  assert.equal(page.element('example').value, 'socrates');
  assert.equal(page.autosave(), draft);
});

test('custom edits remain exact source snapshots, including obsolete syntax', async () => {
  const legacy = await readFile(new URL('tests/fixtures/legacy-socrates.pl', root), 'utf8');
  const source = legacy + '\n% My own changes: Élodie\n';
  const page = await playground({ hash: `#${new URLSearchParams({ code: source })}` });
  assert.equal(page.element('source').value, source);
  assert.equal(page.element('example').value, '');
  assert.equal(page.autosave(), null);
  page.element('source').listeners.input();
  assert.equal(page.autosave(), source);
  assert.equal(new URLSearchParams((await page.share()).hash.slice(1)).get('code'), source);
  const restored = await playground({ source });
  assert.equal(restored.element('source').value, source);
  assert.equal(new URLSearchParams((await restored.share()).hash.slice(1)).get('code'), source);
});
