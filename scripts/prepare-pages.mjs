import { cp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const publicSite = 'https://eyereasoner.github.io/eyelight/';

export async function preparePages(destination, basePath = '/eyelight') {
  if (!/^(?:\/[A-Za-z0-9._-]+)*\/*$/.test(basePath)) {
    throw new Error('Pages base path must be empty or a slash-prefixed URL path.');
  }
  basePath = basePath.replace(/\/+$/, '');
  const localLinks = text => text.replaceAll(publicSite, `${basePath}/`);
  const write = (file, text) => writeFile(resolve(destination, file), text);
  await mkdir(destination, { recursive: true });
  for (const file of ['assets', 'examples', 'pkg', 'LICENSE', 'playground.css', 'playground.js', 'playground-worker.js']) {
    await cp(resolve(root, file), resolve(destination, file), { recursive: true });
  }
  for (const directory of ['SPEC', 'playground', '_layouts']) {
    await mkdir(resolve(destination, directory), { recursive: true });
  }

  const playground = localLinks(await readFile(resolve(root, 'playground.html'), 'utf8'));
  await write('playground.html', playground);
  // Preserve the same module, worker, and example URLs under the directory route.
  await write('playground/index.html', playground.replace('<head>', '<head>\n  <base href="../">'));
  const guide = localLinks(await readFile(resolve(root, 'GUIDE.md'), 'utf8'));
  const spec = localLinks(await readFile(resolve(root, 'SPEC.md'), 'utf8'));
  await write('index.md', `---\nlayout: default\ntitle: eyelight guide\n---\n\n${guide}`);
  await write('SPEC/index.md', `---\nlayout: default\ntitle: eyelight specification\n---\n\n${spec}`);
  await cp(resolve(root, 'scripts/pages-layout.html'), resolve(destination, '_layouts/default.html'));
  await write('_config.yml', JSON.stringify({
    title: 'eyelight', baseurl: basePath, markdown: 'kramdown',
    kramdown: { input: 'GFM', auto_ids: true },
  }, null, 2) + '\n');
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  await preparePages(resolve(root, '_pages'), process.env.PAGES_BASE_PATH ?? '/eyelight');
  console.log('Website prepared in _pages. Render with Jekyll to produce _site.');
}
