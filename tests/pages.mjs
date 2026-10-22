import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { preparePages } from '../scripts/prepare-pages.mjs';

const root = new URL('../', import.meta.url);

test('website stages rendered documentation and every playground dependency', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'eyelight-pages-'));
  try {
    await preparePages(directory);
    for (const file of ['index.md', 'SPEC/index.md', 'playground.html', 'playground/index.html',
      'playground.css', 'playground.js', 'playground-worker.js', 'assets/eyelight.svg', 'assets/docs.css',
      'pkg/eyelight.js', 'pkg/eyelight_bg.wasm', 'examples/socrates.eye', 'examples/paths.eye', 'examples/family.eye',
      'examples/values.eye', 'examples/rdf.eye', 'LICENSE', '_layouts/default.html']) {
      assert.ok((await stat(join(directory, file))).isFile(), file);
    }
    for (const file of ['index.md', 'SPEC/index.md']) {
      const document = await readFile(join(directory, file), 'utf8');
      assert.match(document, /^---\nlayout: default\n/);
      assert.doesNotMatch(document, /\]\([^)]*\.md(?:#|\))/);
    }
    assert.match(await readFile(join(directory, 'index.md'), 'utf8'), /\]\(\/eyelight\/SPEC\/#8-verification\)/);
    const playground = await readFile(join(directory, 'playground/index.html'), 'utf8');
    assert.match(playground, /<base href="\.\.\/">/);
    assert.match(playground, /href="\/eyelight\/SPEC\/"/);
    assert.doesNotMatch(playground, /href="[^"]*\.md(?:#|")/);
    assert.deepEqual(await readFile(join(directory, 'pkg/eyelight_bg.wasm')), await readFile(new URL('pkg/eyelight_bg.wasm', root)));
    for (const directoryName of ['src', 'target', '.github']) {
      await assert.rejects(stat(join(directory, directoryName)), { code: 'ENOENT' });
    }
  } finally { await rm(directory, { recursive: true, force: true }); }
});

test('documentation and playground links honor root and nested Pages prefixes', async () => {
  for (const base of ['', '/preview/eyelight/']) {
    const directory = await mkdtemp(join(tmpdir(), 'eyelight-prefix-'));
    try {
      await preparePages(directory, base);
      const normalized = base.replace(/\/+$/, '');
      assert.equal(JSON.parse(await readFile(join(directory, '_config.yml'), 'utf8')).baseurl, normalized);
      for (const file of ['index.md', 'SPEC/index.md', 'playground.html']) {
        const text = await readFile(join(directory, file), 'utf8');
        assert.doesNotMatch(text, /https:\/\/eyereasoner.github.io\/eyelight\//);
      }
      assert.ok((await readFile(join(directory, 'playground.html'), 'utf8')).includes(`href="${normalized}/SPEC/"`));
    } finally { await rm(directory, { recursive: true, force: true }); }
  }
});

test('committed browser package matches Cargo and performs C1–C9 checking', async () => {
  const { initSync, runProgram, version } = await import('../pkg/eyelight.js');
  initSync({ module: await readFile(new URL('pkg/eyelight_bg.wasm', root)) });
  const cargo = await readFile(new URL('Cargo.toml', root), 'utf8');
  assert.equal(version(), /^version\s*=\s*"([^"]+)"/m.exec(cargo)[1]);
  const source = await readFile(new URL('examples/socrates.eye', root), 'utf8');
  const report = runProgram(source, 'checked', '');
  assert.match(report, /^checked: 2 steps, 1 claim/);
  for (let condition = 1; condition <= 9; condition++) assert.match(report, new RegExp(`C${condition} `));
});
