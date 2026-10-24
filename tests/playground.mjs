import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import vm from 'node:vm';

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
  const script = new vm.Script(source.replace("import('./pkg/whye.js')", 'Promise.resolve(mockEngine)'), { filename: fileURLToPath(new URL('playground-worker.js', root)) });
  context.mockEngine = { default: async () => {}, version: () => '0.1.0', runProgram: (source, mode, proof) => {
    if (source === 'bad') throw new Error('specific failure');
    return `${source}/${mode}/${proof}`;
  } };
  script.runInContext(context);
  await context.self.onmessage({ data: { kind: 'init' } });
  assert.equal(messages[0].version, '0.1.0');
  await context.self.onmessage({ data: { kind: 'run', id: 2, source: 'hello', mode: 'proof', proof: '' } });
  assert.equal(messages[1].output, 'hello/proof/');
  await context.self.onmessage({ data: { kind: 'run', id: 3, source: 'bad' } });
  assert.equal(messages[2].kind, 'error');
  assert.match(messages[2].message, /specific failure/);
});
