// Retain fingerprints of published examples so saved copies can load current sources.
import { createHash } from 'node:crypto';
import { readFile, readdir, writeFile } from 'node:fs/promises';

const directory = new URL('../examples/', import.meta.url);
const file = new URL('index.json', directory);
let index = {};
try { index = JSON.parse(await readFile(file, 'utf8')); }
catch (error) { if (error.code !== 'ENOENT') throw error; }
const names = new Set();
for (const name of await readdir(directory)) {
  if (!name.endsWith('.pl')) continue;
  const source = (await readFile(new URL(name, directory), 'utf8')).replace(/\r\n?/g, '\n');
  const hash = createHash('sha256').update(source).digest('hex');
  names.add(name.slice(0, -3));
  index[hash] = name.slice(0, -3);
}
index = Object.fromEntries(Object.entries(index).filter(([, name]) => names.has(name)).sort(([a], [b]) => a.localeCompare(b)));
await writeFile(file, JSON.stringify(index, null, 2) + '\n');
console.log(`Indexed ${names.size} examples and ${Object.keys(index).length} source fingerprints.`);
