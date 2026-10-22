const $ = id => document.getElementById(id);
const modes = new Set(['answers', 'explain', 'proof', 'checked', 'check']);
const examples = new Set(['socrates', 'paths', 'family', 'values', 'rdf', 'arithmetic', 'negation', 'aggregation', 'witnesses', 'orders']);
const storageKey = 'eyelight.playground.v1';
let worker = null;
let ready = null;
let sequence = 0;
let busy = false;
let exampleSource = null;
let saveTimer;
let cancelReady = null;
let runningState = null;

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
  return { source: $('source').value, mode: $('mode').value, proof: $('saved-proof').value };
}
function save() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    try { localStorage.setItem(storageKey, JSON.stringify(snapshot())); } catch { /* Storage can be unavailable. */ }
  }, 200);
}
function syncMode() {
  $('saved-proof-panel').hidden = $('mode').value !== 'check';
}
function restore(state) {
  if (typeof state?.source !== 'string') return false;
  $('source').value = state.source;
  $('mode').value = modes.has(state.mode) ? state.mode : 'answers';
  $('saved-proof').value = typeof state.proof === 'string' ? state.proof : '';
  $('example').value = '';
  syncMode();
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
$('saved-proof').addEventListener('input', save);
$('mode').addEventListener('change', () => { syncMode(); save(); });
$('source').addEventListener('keydown', event => {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') { event.preventDefault(); if (!busy) $('run').click(); }
});
async function loadExample(name) {
  if (!examples.has(name)) return;
  if (busy) stop();
  $('source').disabled = true; $('example').disabled = true; $('run').disabled = true;
  try {
    const response = await fetch(new URL(`./examples/${name}.eye`, import.meta.url));
    if (!response.ok) throw new Error(`Cannot load example ${name}: HTTP ${response.status}.`);
    const source = await response.text();
    restore({ source, mode: 'answers', proof: '' });
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
  if (exampleSource && state.source === exampleSource.source && state.mode === 'answers' && !state.proof) {
    params.set('example', exampleSource.name);
  } else {
    params.set('code', state.source);
    params.set('mode', state.mode);
    if (state.proof) params.set('proof', state.proof);
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
  $('output').textContent = 'Your first answer is one click away.\n\nPress Run to follow your facts and rules.';
  try {
    const params = new URLSearchParams(location.hash.slice(1));
    if (params.has('code')) restore({ source: params.get('code'), mode: params.get('mode'), proof: params.get('proof') });
    else if (params.has('example')) {
      if (!examples.has(params.get('example'))) throw new Error('Unknown shared example. Choose an example above.');
      await loadExample(params.get('example'));
    } else {
      let saved;
      try { saved = JSON.parse(localStorage.getItem(storageKey)); } catch { /* No usable autosave. */ }
      if (!restore(saved)) await loadExample('socrates');
    }
    syncMode();
    status('Ready when you are.');
    // Loading shared state never executes it automatically.
  } catch (error) { status(error.message, true); }
}
window.addEventListener('hashchange', () => { void initialise(); });
await initialise();
