const $ = id => document.getElementById(id);
const examples = new Set(['abstract-interpretation', 'access-control-policy', 'ackermann', 'age', 'aggregation', 'animals', 'arithmetic', 'battery-dispatch', 'bayes-diagnosis', 'beam-deflection', 'binomial-vandermonde', 'braking-distance', 'buck-converter-design', 'collatz', 'complex', 'control-system', 'critical-path-schedule', 'data-quality-audit', 'deep-taxonomy-10', 'deep-taxonomy-100', 'deep-taxonomy-1000', 'deep-taxonomy-10000', 'deep-taxonomy-100000', 'delfour', 'dijkstra', 'dog-license', 'easter', 'electrical-rc-filter', 'explainable-ev-depot-configuration', 'expression-eval', 'family', 'family-cousins', 'fft8-numeric', 'fibonacci', 'field-nitrogen-balance', 'flute-puzzle', 'french-cities', 'friends', 'good-cobbler', 'gps', 'gray-code-counter', 'hamming-code', 'hanoi', 'interval-relations', 'inventory', 'job-shop-scheduling', 'knapsack-optimization', 'lagrange-cosets', 'least-squares-regression', 'list-collection', 'lists', 'matrix-multiplication', 'modular-exponentiation', 'negation', 'newton-raphson', 'observability-log-correlation', 'orbital-transfer-design', 'orders', 'padovan', 'paraconsistent-animals', 'path-discovery', 'paths', 'peano', 'peasant', 'pell-equation', 'permissions', 'policy-risk', 'polynomial', 'queens', 'radioactive-decay', 'rdf', 'research-portal', 'sbom-vulnerability-response', 'schema-inference', 'service-outage-recovery', 'shortest-path', 'sieve', 'socrates', 'state-transitions', 'statistics-summary', 'symbolic-derivative', 'term-structure', 'text-processing', 'turing', 'type-inference', 'unification', 'values', 'vector-similarity', 'weighted-interval-scheduling', 'wind-turbine', 'witnesses']);
const storageKey = 'wheye.playground.v1';
let worker = null;
let ready = null;
let sequence = 0;
let busy = false;
let exampleSource = null;
let saveTimer;
let cancelReady = null;
let runningState = null;
let exampleIndex = null;

async function identifyExample(source) {
  if (!globalThis.crypto?.subtle) return null;
  try {
    exampleIndex ??= fetch(new URL('./examples/index.json', import.meta.url), { cache: 'no-cache' })
      .then(response => {
        if (!response.ok) throw new Error('Cannot load the example index.');
        return response.json();
      });
    const [index, digest] = await Promise.all([
      exampleIndex,
      crypto.subtle.digest('SHA-256', new TextEncoder().encode(source.replace(/\r\n?/g, '\n'))),
    ]);
    const hash = [...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, '0')).join('');
    const name = index[hash];
    return examples.has(name) ? name : null;
  } catch {
    exampleIndex = null;
    return null;
  }
}

async function restoreSource(source) {
  const name = await identifyExample(source);
  if (name) await loadExample(name);
  else {
    exampleSource = null;
    restore({ source });
  }
  return name;
}

function status(message, error = false) {
  $('status').textContent = message;
  $('output').classList.toggle('error', error);
}
function working(value) {
  busy = value;
  $('run').disabled = value;
  $('stop').disabled = !value;
}
function snapshot() {
  return { source: $('source').value };
}
function save() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    try { localStorage.setItem(storageKey, JSON.stringify(snapshot())); } catch { /* Storage can be unavailable. */ }
  }, 200);
}
function restore(state) {
  if (typeof state?.source !== 'string') return false;
  $('source').value = state.source;
  $('example').value = '';
  return true;
}
function ensureWorker() {
  if (ready) return ready;
  worker = new Worker(new URL('./playground-worker.js', import.meta.url), { type: 'module' });
  const current = worker;
  ready = new Promise((resolve, reject) => {
    cancelReady = reject;
    current.onmessage = ({ data }) => {
      if (current !== worker) return;
      if (data.kind === 'ready') {
        $('version').textContent = `v${data.version}`;
        cancelReady = null;
        resolve(current);
      } else if (data.kind === 'fatal') {
        reject(new Error(`Cannot load the browser engine: ${data.message}. See the local setup in the guide.`));
        current.terminate(); worker = null; ready = null;
      } else if (data.id === sequence) {
        working(false);
        if (data.kind === 'result') {
          $('output').textContent = data.output || 'No new facts were derived.';
          status(JSON.stringify(snapshot()) === runningState ? 'Complete.' : 'Complete for the program at Run. Run again to use your edits.');
        } else {
          $('output').textContent = data.message;
          status('The program could not complete.', true);
        }
      }
    };
    current.onerror = event => {
      if (current !== worker) return;
      const error = new Error(event.message || 'The reasoning worker could not start. Serve this page over HTTP.');
      reject(error);
      current.terminate(); worker = null; ready = null;
      if (busy) { working(false); $('output').textContent = error.message; status('Worker error.', true); }
    };
    current.postMessage({ kind: 'init' });
  });
  return ready;
}
function stop() {
  sequence++;
  cancelReady?.(new Error('Stopped.')); cancelReady = null;
  worker?.terminate(); worker = null; ready = null;
  working(false);
  status('Stopped.');
  $('output').textContent = 'Run stopped. Edit the program or press Run to start again.';
}

$('run').addEventListener('click', async () => {
  const id = ++sequence;
  const state = snapshot();
  runningState = JSON.stringify(state);
  working(true); status('Running…');
  $('output').textContent = 'Following the rules…';
  try {
    const current = await ensureWorker();
    if (id === sequence) current.postMessage({ kind: 'run', id, ...state });
  } catch (error) {
    if (id !== sequence) return;
    working(false); $('output').textContent = error.message; status('Unable to run.', true);
  }
});
$('stop').addEventListener('click', stop);
$('source').addEventListener('input', () => { $('example').value = ''; save(); });
$('source').addEventListener('keydown', event => {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') { event.preventDefault(); if (!busy) $('run').click(); }
});
async function loadExample(name) {
  if (!examples.has(name)) return;
  if (busy) stop();
  $('source').disabled = true; $('example').disabled = true; $('run').disabled = true;
  try {
    const response = await fetch(new URL(`./examples/${name}.pl`, import.meta.url), { cache: 'no-cache' });
    if (!response.ok) throw new Error(`Cannot load example ${name}: HTTP ${response.status}.`);
    const source = await response.text();
    restore({ source });
    exampleSource = { name, source };
    $('example').value = name;
    status('Example loaded. Press Run.');
  } finally {
    $('source').disabled = false; $('example').disabled = false; $('run').disabled = busy;
  }
}
$('example').addEventListener('change', async () => {
  try { await loadExample($('example').value); save(); } catch (error) { status(error.message, true); }
});
$('copy').addEventListener('click', async () => {
  try { await navigator.clipboard.writeText($('output').textContent); status('Result copied.'); }
  catch { status('Select the result text to copy it.'); }
});
$('share').addEventListener('click', async () => {
  const state = snapshot();
  const url = new URL(location.href);
  const params = new URLSearchParams();
  const name = exampleSource && state.source === exampleSource.source
    ? exampleSource.name : await identifyExample(state.source);
  if (name) {
    params.set('example', name);
  } else {
    params.set('code', state.source);
  }
  url.hash = params.toString();
  $('share-url').value = url.href;
  $('share-box').hidden = false;
  try { await navigator.clipboard.writeText(url.href); status('Share link copied.'); }
  catch { $('share-url').focus(); $('share-url').select(); status('Copy the share link above.'); }
});
$('close-share').addEventListener('click', () => { $('share-box').hidden = true; });

async function initialise() {
  if (busy) stop();
  clearTimeout(saveTimer);
  $('source').disabled = true; $('example').disabled = true; $('run').disabled = true;
  $('output').textContent = 'Your first answer is one click away.\n\nPress Run to follow your facts and rules.';
  try {
    const params = new URLSearchParams(location.hash.slice(1));
    if (params.has('code')) {
      const name = await restoreSource(params.get('code'));
      if (name) {
        const url = new URL(location.href);
        url.hash = new URLSearchParams({ example: name }).toString();
        history.replaceState(null, '', url);
      }
    }
    else if (params.has('example')) {
      if (!examples.has(params.get('example'))) throw new Error('Unknown shared example. Choose an example above.');
      await loadExample(params.get('example'));
    } else {
      let saved;
      try { saved = JSON.parse(localStorage.getItem(storageKey)); } catch { /* No usable autosave. */ }
      if (typeof saved?.source === 'string') {
        const name = await restoreSource(saved.source);
        if (name && saved.source !== $('source').value) save();
      } else await loadExample('socrates');
    }
    status('Ready when you are.');
    // Loading shared state never executes it automatically.
  } catch (error) { status(error.message, true); }
  finally {
    $('source').disabled = false; $('example').disabled = false; $('run').disabled = busy;
  }
}
window.addEventListener('hashchange', () => { void initialise(); });
await initialise();
