// Real desktop end-to-end drive of a built Hub (dev build or the installed one) through WebView2's DevTools port.
//   node tools/hub_native_e2e.mjs <hub.exe> [--remote-catalog] [--report DIR]
// --remote-catalog: products come through the download path (a signed file:// catalog made by release/tools/publish_release.py).
// Everything runs on disposable roots and a FAKE user home (a foreign ~/.claude.json to be preserved). The
// Hub is started with Python removed from PATH, so passing proves products run without a system Python.
import { spawn, spawnSync } from 'node:child_process';
import { cpSync, readdirSync, mkdtempSync, mkdirSync, writeFileSync, existsSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname, delimiter } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const args = process.argv.slice(2);
const exe = resolve(args.find(a => !a.startsWith('--')) ?? '');
const remote = args.includes('--remote-catalog');
const brainOnly = args.includes('--only-brain'); // targeted rerun: skip the Orchestrator/runtime scenarios
const eco = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const tools = join(eco, 'release', 'tools');
const reportDir = resolve(args.includes('--report') ? args[args.indexOf('--report') + 1] : join(eco, '.campaign-data', 'master-plan', 'reports', 'hub-native'));
mkdirSync(reportDir, { recursive: true });
if (!existsSync(exe)) { console.error('exe not found:', exe); process.exit(2); }

const work = mkdtempSync(join(tmpdir(), 'hub-native-'));
const dirs = { catalog: join(work, 'catalog'), managed: join(work, 'managed'), data: join(work, 'data'), home: join(work, 'home'),
  brainApp: join(work, 'brain-app'), brainData: join(work, 'brain-data'), vaults: join(work, 'vaults') };
Object.values(dirs).forEach(d => mkdirSync(d, { recursive: true }));
mkdirSync(join(dirs.home, '.gemini'), { recursive: true });
const foreign = '{"mcpServers":{"someone-elses":{"command":"x","args":[]}},"theme":"dark"}';
writeFileSync(join(dirs.home, '.claude.json'), foreign);

// Guard: the run must not change the REAL AI-client configuration (tools/real_env_guard.py compares before and after).
const guardFile = join(work, 'real-env-before.json');
spawnSync('python', ['-B', join(tools, 'real_env_guard.py'), 'snapshot', guardFile], { encoding: 'utf8' });
const py = (...a) => { const r = spawnSync('python', ['-B', ...a], { encoding: 'utf8' }); if (r.status !== 0) { console.error(r.stderr); process.exit(2); } return r.stdout.trim(); };
const dist = py(join(tools, 'build_brain_dist.py')).split(/\r?\n/).pop();
py(join(tools, 'build_bundle.py'), 'orchestrator', '--out', dirs.catalog);
py(join(tools, 'build_bundle.py'), 'secondbrain-dist', '--dist', dist, '--out', dirs.catalog);
py(join(tools, 'build_runtime.py'), '--out', dirs.catalog);
const served = join(work, 'served');
if (remote) py(join(tools, 'publish_release.py'), 'local-catalog', ...readdirSync(dirs.catalog).flatMap(name => ['--bundle', join(dirs.catalog, name)]), '--out', served);

const port = 9333;
const pathWithoutPython = process.env.PATH.split(delimiter).filter(p => !/python|WindowsApps/i.test(p)).join(delimiter);
const env = { ...process.env, PATH: pathWithoutPython, RESPECTED_HUB_TEST_HOME: dirs.home,
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`, WEBVIEW2_USER_DATA_FOLDER: join(work, 'webview'),
  RESPECTED_HUB_MANAGED_ROOT: dirs.managed, RESPECTED_ORCHESTRATOR_DATA_ROOT: dirs.data,
  RESPECTED_HUB_BRAIN_APP_DIR: dirs.brainApp, RESPECTED_HUB_BRAIN_DATA_DIR: dirs.brainData, PYTHONDONTWRITEBYTECODE: '1' };
// development builds trust the local dev signing key only when told where it is (an installed dev build has dev-trust.pub beside it)
env.RESPECTED_HUB_DEV_TRUST = join(eco, '..', '.build-cache', 'signing', 'dev-ed25519.pub');
if (remote) env.RESPECTED_HUB_CATALOG_URL = pathToFileURL(join(served, 'catalog.json')).href; // cache: <managed>/catalog
else env.RESPECTED_HUB_CATALOG_ROOT = dirs.catalog;
const intent = args.find(a => a.startsWith('--intent='))?.slice(9); // product installers launch Hub with --install=<product>
const child = spawn(exe, intent ? [`--install=${intent}`] : [], { env, stdio: 'ignore' });

const results = [];
const check = (name, ok, detail = '') => { results.push({ name, ok, detail }); console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ' — ' + detail : ''}`); };
const sleep = ms => new Promise(r => setTimeout(r, ms));

async function target() {
  for (let i = 0; i < 80; i++) {
    try { const list = await (await fetch(`http://127.0.0.1:${port}/json`)).json(); const page = list.find(t => t.type === 'page'); if (page) return page; } catch { /* not up yet */ }
    await sleep(500);
  }
  throw new Error('WebView2 devtools target never appeared');
}

let ws, seq = 0; const pending = new Map(); const events = [];
async function connect(url) {
  ws = new WebSocket(url);
  await new Promise((ok, bad) => { ws.onopen = ok; ws.onerror = bad; });
  ws.onmessage = event => { const m = JSON.parse(event.data); if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); } else if (m.method === 'Runtime.exceptionThrown') events.push('EXC ' + JSON.stringify(m.params.exceptionDetails.exception?.description ?? m.params.exceptionDetails.text).slice(0, 300)); else if (m.method === 'Runtime.consoleAPICalled' && ['error', 'warning'].includes(m.params.type)) events.push(m.params.type + ' ' + m.params.args.map(a => a.value ?? a.description).join(' ').slice(0, 300)); else if (m.method === 'Page.frameNavigated' && !m.params.frame.parentId) events.push('NAVIGATED ' + m.params.frame.url); };
}
const send = (method, params = {}) => new Promise(ok => { const id = ++seq; pending.set(id, ok); ws.send(JSON.stringify({ id, method, params })); });
async function ev(expression) {
  const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  if (r.result?.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails.exception?.description ?? r.result.exceptionDetails).slice(0, 200) + ' IN ' + expression.slice(0, 160));
  return r.result?.result?.value;
}
async function shot(name) {
  const r = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(join(reportDir, `${name}.png`), Buffer.from(r.result.data, 'base64'));
}
async function waitFor(expression, label, ms = 60000, soft = false) {
  const end = Date.now() + ms;
  while (Date.now() < end) { try { if (await ev(expression)) return true; } catch { /* page navigating */ } await sleep(500); }
  check(label, false, 'timed out');
  try { await shot('timeout-' + label.replace(/\W+/g, '-').slice(0, 40)); writeFileSync(join(reportDir, 'timeout-' + label.replace(/\W+/g, '-').slice(0, 40) + '.txt'), await ev('document.body.innerText')); } catch { /* best effort */ }
  if (!soft) throw new Error(`step failed: ${label} (timed out)`); // fail fast: later steps cannot succeed and would only wait out their own timeouts
  return false;
}
const clickText = (text, scope = 'document') => ev(`(()=>{const root=${scope};const b=[...root.querySelectorAll('button')].find(x=>x.textContent.trim().includes(${JSON.stringify(text)})&&!x.disabled);if(!b)return false;b.click();return true})()`);
const card = name => `[...document.querySelectorAll('article.lib-item')].find(a=>a.textContent.includes(${JSON.stringify(name)}))`;
const inCard = (name, label, exact = true) => ev(`(()=>{const a=${card(name)};const b=[...a.querySelectorAll('button')].find(x=>${exact ? `x.textContent.trim()===${JSON.stringify(label)}` : `x.textContent.includes(${JSON.stringify(label)})`}&&!x.disabled);if(!b)return false;b.click();return true})()`);
const cardHas = (name, text) => `(${card(name)})?.textContent.includes(${JSON.stringify(text)})`;
const setInput = (selector, value) => ev(`(()=>{const el=${selector};const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;set.call(el,${JSON.stringify(value)});el.dispatchEvent(new Event('input',{bubbles:true}));return true})()`);
const rail = label => clickText(label, 'document.querySelector(".rail")');
const readJson = path => JSON.parse(readFileSync(path, 'utf8'));

async function install(name, key, minutes = 3) {
  await inCard(name, 'Kur');
  await waitFor(cardHas(name, 'Onayla ve uygula'), `${name}: install plan shown`);
  await inCard(name, 'Onayla ve uygula', false);
  const ok = await waitFor(cardHas(name, 'Kurulu ve doğrulandı'), `${name}: installed and verified`, minutes * 60000, true);
  check(`${name}: current.json written`, existsSync(join(dirs.managed, key, 'current.json')));
  return ok;
}

try {
  await connect((await target()).webSocketDebuggerUrl);
  await send('Page.enable'); await send('Runtime.enable');
  await waitFor(`document.body.innerText.includes('Respected')`, 'app window renders');
  if (intent) { // product-download scenario: Hub must open on that product's install page, then stop.
    await waitFor(`document.querySelector('.crumbs')?.textContent.includes(${JSON.stringify(intent === 'secondbrain' ? 'Kurulum' : 'Uygulamalarım')})`, `${intent} download opens its install page`, 30000);
    await shot('intent-' + intent);
    check('other product not installed or forced', !existsSync(join(dirs.managed, intent === 'secondbrain' ? 'orchestrator' : 'secondbrain', 'current.json')));
    child.kill(); process.exit(results.every(r => r.ok) ? 0 : 1);
  }
  check('running as the native desktop app', await ev(`'__TAURI_INTERNALS__' in window`));
  check('Python is not on the PATH of this Hub', !spawnSync('python', ['--version'], { env, encoding: 'utf8' }).stdout);

  await rail('Uygulamalarım');
  await waitFor(`document.querySelector('.page-head h1')?.textContent==='Uygulamalarım'&&document.querySelectorAll('article.lib-item').length===2`, 'My Apps lists both products');
  await waitFor(cardHas('SecondBrain', 'Kurulu değil'), 'SecondBrain starts not installed');
  check('Python runtime component is offered', await ev(`document.body.innerText.includes('Python çalışma ortamı')`));
  await shot('01-my-apps-before');

  if (!brainOnly) { // Orchestrator: the install also provisions the Hub's own Python runtime first.
  await inCard('Orchestrator', 'Kur');
  await waitFor(cardHas('Orchestrator', 'Önce Hub'), 'Orchestrator plan announces the Python runtime step');
  await shot('02-plan-orchestrator');
  await inCard('Orchestrator', 'Onayla ve uygula', false);
  await waitFor(cardHas('Orchestrator', 'Kurulu ve doğrulandı'), 'Orchestrator installed and verified', 240000);
  check('runtime and orchestrator recorded', existsSync(join(dirs.managed, 'runtime', 'current.json')) && existsSync(join(dirs.managed, 'orchestrator', 'current.json')));
  await waitFor(`document.querySelector('.sys-component')?.textContent.includes('Kurulu ve doğrulandı')`, 'runtime component shows installed');
  await waitFor(`(()=>{const a=${card('Orchestrator')};return !!a&&!a.textContent.includes('Ürüne ulaşılamadı')&&/v0\\.1\\.0/.test(a.textContent)})()`, 'Orchestrator answers over RPP on the managed runtime', 60000);
  await shot('03-orchestrator-installed');
  }

  // SecondBrain: the wizard collects vault + integrations; the product's installer does the work.
  await inCard('SecondBrain', 'Kur');
  await waitFor(`document.querySelector('.wizard .steps li.current')?.textContent.includes('Kimlik')&&document.body.innerText.includes('SecondBrain\\'i kuralım')`, 'install wizard opens at Identity');
  check('wizard steps are Identity, Memory, Sources, Permissions, Finish', await ev(`[...document.querySelectorAll('.wizard .steps li')].map(l=>l.textContent.replace(/^\\d/,'')).join('|')`) === 'Kimlik|Hafıza|Kaynaklar|İzinler|Onay');
  await setInput(`document.querySelector('.wizard input[placeholder="ör. Ayşe"]')`, 'Test');
  await setInput(`document.querySelector('.wizard input[placeholder="ör. Jarvis"]')`, 'Jarvis');
  await clickText('Devam');
  await setInput(`[...document.querySelectorAll('.vault-create label')].find(l=>l.textContent.startsWith('Klasör')).querySelector('input')`, dirs.vaults);
  await setInput(`[...document.querySelectorAll('.vault-create label')].find(l=>l.textContent.includes('Kasa klasörünün adı')).querySelector('input')`, 'Brain Vault');
  await ev(`document.querySelectorAll('input[name=capture]')[1].click()`);  // manual capture
  await clickText('Devam');
  const hooks = `document.querySelector('[aria-label="Hafıza kaynağı AI araçları"]')`;
  const mcpList = `document.querySelector('[aria-label="MCP erişimi verilecek AI araçları"]')`;
  await waitFor(`!!${hooks}&&${hooks}.querySelector('[data-client="claude"] input')&&!${hooks}.querySelector('[data-client="claude"] input').disabled&&${hooks}.textContent.includes('Bu bilgisayarda bulunamadı')`, 'Sources step lists clients and detects which exist', 90000);
  check('undetected clients cannot be selected, detected ones can', await ev(`${hooks}.querySelector('[data-client="codex"] input').disabled&&!${hooks}.querySelector('[data-client="gemini"] input').disabled`));
  await ev(`${hooks}.querySelector('[data-client="claude"] input').click()`);
  await waitFor(`[...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='Devam'&&!b.disabled)&&!document.body.innerText.includes('Plan hazırlanıyor')`, 'plan for the Sources choice is ready', 90000);
  await clickText('Devam');
  await waitFor(`!!${mcpList}`, 'Permissions step shows per-client MCP switches');
  await ev(`${mcpList}.querySelector('[data-client="claude"] input').click()`);
  await waitFor(`document.querySelectorAll('.plan-changes li').length>0&&[...document.querySelectorAll('.plan-changes li')].some(l=>l.textContent.includes('.claude.json')&&l.textContent.includes('Değiştirilir'))`, 'integration plan lists the exact files (.claude.json would be modified)', 90000);
  check('plan names only the chosen client (no Gemini/Codex file)', !(await ev(`document.querySelector('.plan-changes').textContent.includes('.gemini')||document.querySelector('.plan-changes').textContent.includes('.codex')`)));
  check('planning wrote nothing', readFileSync(join(dirs.home, '.claude.json'), 'utf8') === foreign && !existsSync(join(dirs.vaults, 'Brain Vault')));
  await shot('04-integration-plan');
  await clickText('Devam');
  await waitFor(`document.body.innerText.includes('Onayla ve kur')`, 'approval step shows');
  await shot('05-approval');
  await clickText('Onayla ve kur');
  const brainOk = await waitFor(`document.querySelector('.ws-banner')?.textContent.includes('v0.0.1')&&!!document.querySelector('.wizard')&&document.body.innerText.includes('Karşılama')&&document.body.innerText.includes('Kayıtlı kasa')`, 'SecondBrain installed; settings wizard follows', 420000, true);
  check('app launcher installed by the product', existsSync(join(dirs.brainApp, 'respectedbrain.exe')));
  check('vault created with the template', existsSync(join(dirs.vaults, 'Brain Vault', '.respected.json')));
  const claude = readJson(join(dirs.home, '.claude.json'));
  check('foreign MCP entry preserved, Respected entry added', !!claude.mcpServers?.['someone-elses'] && !!claude.mcpServers?.['respected-vault'] && claude.theme === 'dark');
  check('only the chosen client was configured (Claude hooks yes, Gemini no)', existsSync(join(dirs.home, '.claude', 'settings.json')) && !existsSync(join(dirs.home, '.gemini', 'config', 'hooks.json')) && !existsSync(join(dirs.home, '.gemini', 'settings.json')));
  const stored = readJson(join(dirs.brainData, 'config.json'));
  check('client choices and capture mode are stored by the product', JSON.stringify(stored.integrations.clients) === '["claude"]' && JSON.stringify(stored.integrations.mcp_clients) === '["claude"]' && stored.preferences.capture_mode === 'manual', JSON.stringify(stored.integrations) + JSON.stringify(stored.preferences));
  const walk = dir => readdirSync(dir, { withFileTypes: true }).flatMap(e => e.isDirectory() ? (e.name.startsWith('.') ? [] : walk(join(dir, e.name))) : [join(dir, e.name)]);
  check('assistant name was written into the vault template', walk(join(dirs.vaults, 'Brain Vault')).filter(f => f.endsWith('.md')).some(f => readFileSync(f, 'utf8').includes('Jarvis')));
  if (brainOk) await shot('06-brain-installed');
  const probe = await ev(`window.__TAURI_INTERNALS__.invoke('rpp_request',{product:'secondbrain',op:'manifest',payload:null,operationId:null}).then(r=>JSON.stringify({ok:r.ok,state:r.result?.install_state,roots:r.result?.roots,err:r.error}))`);
  console.log('manifest probe:', probe);
  if (args.includes('--stop-after-install')) throw new Error('stopped after install (diagnostic run)');

  // Settings wizard (installed): integrations panel switches the MCP entry off; the product restores the config.
  await clickText('Devam'); await clickText('Devam');
  await waitFor(`document.body.innerText.includes('AI araç entegrasyonları')&&document.querySelectorAll('.step-body [data-client]').length>=10`, 'integrations panel lists clients for the installed product');
  await waitFor(`!document.querySelector('[aria-label="Hafıza kaynağı AI araçları"] [data-client="gemini"] input').checked`, 'panel adopts the stored per-client choice from the product', 90000);
  check('panel shows the stored per-client choice (Claude on, Gemini off)', await ev(`document.querySelector('[aria-label="Hafıza kaynağı AI araçları"] [data-client="claude"] input').checked&&!document.querySelector('[aria-label="Hafıza kaynağı AI araçları"] [data-client="gemini"] input').checked`));
  await ev(`document.querySelector('[aria-label="MCP erişimi verilecek AI araçları"] [data-client="claude"] input').click()`);
  await waitFor(`!!document.querySelector('.plan-changes')`, 'switch-off plan listed', 90000);
  await clickText('Entegrasyonları uygula');
  await waitFor(`document.body.innerText.includes('Entegrasyonlar uygulandı')`, 'integration change applied and verified', 300000);
  const after = readJson(join(dirs.home, '.claude.json'));
  check('MCP entry removed, foreign entry intact', !after.mcpServers?.['respected-vault'] && !!after.mcpServers?.['someone-elses']);

  // Memory page (replaces the removed local panel): quick capture -> search finds it -> reader opens it; the note is a new file in the Dump inbox.
  await rail('SecondBrain');
  await clickText('Hafıza', 'document.querySelector(".ws-tabs")');
  await waitFor(`!!document.querySelector('[aria-label="Hızlı not yakala"]')`, 'Memory page shows search, capture and briefing sections', 60000);
  await ev(`(()=>{const el=document.querySelector('[aria-label="Not içeriği"]');Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(el,'NATIVE-MARMALADE-77 ilk kahvaltı notu');el.dispatchEvent(new Event('input',{bubbles:true}));return true})()`);
  await clickText('Kaydet', `document.querySelector('[aria-label="Hızlı not yakala"]')`);
  await waitFor(`document.querySelector('[aria-label="Hızlı not yakala"]').textContent.includes('Kaydedildi')`, 'quick capture reports the saved note', 90000);
  const dump = join(dirs.vaults, 'Brain Vault', '📥 000-Inbox', 'Dump');
  check('quick capture created exactly one new note in the Dump inbox', existsSync(dump) && readdirSync(dump).filter(f => f.endsWith('.md')).length === 1);
  await setInput(`document.querySelector('[aria-label="Arama sorgusu"]')`, 'NATIVE-MARMALADE-77');
  await clickText('Ara', `document.querySelector('[aria-label="Not ara"]')`);
  await waitFor(`document.querySelector('[aria-label="Not ara"]').textContent.includes('Dump')`, 'search finds the captured note', 90000);
  await clickText('Oku', `document.querySelector('[aria-label="Not ara"]')`);
  await waitFor(`document.querySelector('[aria-label="Not oku"] pre')?.textContent.includes('ilk kahvaltı notu')`, 'reader shows the note content', 60000);
  await shot('14-memory-page');
  await rail('SecondBrain'); await clickText('Genel bakış', 'document.querySelector(".ws-tabs")');

  // Optional semantic search panel: real runtime + model download into the disposable brain data root, then disable again.
  if (args.includes('--real-semantic')) {
    const sem = `document.querySelector('[aria-label="Anlamsal arama (isteğe bağlı)"]')`;
    await rail('SecondBrain');
    await clickText('Ayarlar', 'document.querySelector(".ws-tabs")');
    await waitFor(`!!${sem}&&${sem}.textContent.includes('Kapalı')`, 'semantic panel shows Off (FTS stays the default)', 90000);
    await shot('12-semantic-off');
    await clickText('Etkinleştir', sem);
    await waitFor(`${sem}.textContent.includes('460 MB')&&${sem}.textContent.includes('Notlarınız bilgisayarınızdan çıkmaz')`, 'semantic confirmation explains size and privacy');
    await clickText('Vazgeç', sem);
    check('declining downloaded nothing', !existsSync(join(dirs.brainData, 'semantic-runtime')));
    // The download itself was verified separately; reuse that verified copy so this run needs no network.
    const seed = args.includes('--seed-semantic') ? resolve(args[args.indexOf('--seed-semantic') + 1]) : null;
    if (seed) cpSync(seed, join(dirs.brainData, 'semantic-runtime'), { recursive: true });
    await clickText('Etkinleştir', sem);
    await clickText('Onayla, indir ve etkinleştir', sem);
    const ready = await waitFor(`${sem}.textContent.includes('Hazır')`, 'semantic panel reaches Ready after download + vector build', 1500000, true);
    await shot('13-semantic-ready');
    check('runtime and model exist in the product data root only', existsSync(join(dirs.brainData, 'semantic-runtime', 'llama', 'llama-server.exe')) && existsSync(join(dirs.brainData, 'semantic-runtime', 'models', 'bge-m3-Q4_K_M.gguf')));
    if (ready) {
      await clickText('Kapat', sem);
      await waitFor(`${sem}.textContent.includes('Kapalı')`, 'semantic panel switched off again');
    }
  }

  // Repair SecondBrain (tampered launcher) through the product's own repairer.
  await rail('Uygulamalarım');
  await waitFor(cardHas('SecondBrain', 'Kurulu ve doğrulandı'), 'SecondBrain shows installed', 60000);
  writeFileSync(join(dirs.brainApp, 'distribution.json'), readFileSync(join(dirs.brainApp, 'distribution.json'), 'utf8') + ' ');
  await ev(`document.querySelector('button[aria-label="Durumu yenile"]').click()`);
  await waitFor(cardHas('SecondBrain', 'Onarım gerekli'), 'tampering detected', 40000);
  await inCard('SecondBrain', 'Onar');
  await waitFor(cardHas('SecondBrain', 'Onayla ve uygula'), 'repair plan shown');
  await inCard('SecondBrain', 'Onayla ve uygula', false);
  await waitFor(cardHas('SecondBrain', 'Kurulu ve doğrulandı'), 'SecondBrain repaired', 420000);
  await shot('07-brain-repaired');

  if (!brainOnly) { // Orchestrator: tamper, repair, product and Hub agree.
  const cur = readJson(join(dirs.managed, 'orchestrator', 'current.json'));
  const init = join(dirs.managed, 'orchestrator', 'versions', cur.version, 'src', 'respected_orchestrator', '__init__.py');
  writeFileSync(init, readFileSync(init, 'utf8') + '\n# tampered\n');
  await ev(`document.querySelector('button[aria-label="Durumu yenile"]').click()`);
  await waitFor(cardHas('Orchestrator', 'Onarım gerekli'), 'Orchestrator tampering detected', 40000);
  await inCard('Orchestrator', 'Onar');
  await waitFor(cardHas('Orchestrator', 'Onayla ve uygula'), 'Orchestrator repair plan');
  await inCard('Orchestrator', 'Onayla ve uygula', false);
  await waitFor(cardHas('Orchestrator', 'Kurulu ve doğrulandı'), 'Orchestrator repaired', 180000);
  check('tampered file restored', !readFileSync(init, 'utf8').includes('tampered'));
  }

  // Uninstall SecondBrain: vault kept, foreign config byte-exact; Activity shows everything.
  writeFileSync(join(dirs.home, '.claude.json'), foreign); // simulate the user's own edit after switch-off for an exact comparison below
  await inCard('SecondBrain', 'Kaldır');
  await waitFor(cardHas('SecondBrain', 'verileriniz'), 'uninstall plan keeps data');
  await inCard('SecondBrain', 'Onayla ve uygula', false);
  await waitFor(cardHas('SecondBrain', 'Kurulu değil'), 'SecondBrain uninstalled', 420000);
  check('program removed, vault kept, foreign config intact', !existsSync(join(dirs.brainApp, 'respectedbrain.exe')) && existsSync(join(dirs.vaults, 'Brain Vault')) && readFileSync(join(dirs.home, '.claude.json'), 'utf8') === foreign);
  await rail('Etkinlik');
  await waitFor(`document.body.innerText.includes('Kurulum işlemleri')&&document.body.innerText.includes('kaldırıldı')&&document.body.innerText.includes('onarıldı')`, 'Activity lists lifecycle operations');
  await shot('08-activity');

  if (!brainOnly) { // Orchestrator settings through RPP apply + readback on the managed runtime.
  await rail('Orchestrator');
  await waitFor(`document.querySelector('.ws-tabs')!==null`, 'Orchestrator workspace opens');
  await clickText('Ayarlar', 'document.querySelector(".ws-tabs")');
  await waitFor(`document.querySelector('details.settings-block summary')!==null`, 'orchestrator settings panel');
  await ev(`document.querySelector('details.settings-block summary').click()`);
  await waitFor(`document.querySelectorAll('.settings-block input[type=number]').length>0`, 'orchestrator settings load');
  await setInput(`document.querySelector('.settings-block input[type=number]')`, '1234');
  await clickText('Uygula', 'document.querySelector(".settings-block")');
  await waitFor(`document.body.innerText.includes('geri okunarak doğrulandı')`, 'orchestrator settings applied and read back', 60000);
  check('orchestrator settings.json holds the new value', existsSync(join(dirs.data, 'settings.json')) && readFileSync(join(dirs.data, 'settings.json'), 'utf8').includes('1234'));
  await shot('09-orchestrator-settings');
  // MCP panel: toggle Claude Code on, check the exact plan, apply, verify config + launcher; then switch off again.
  const mcpSwitch = `(()=>{const l=[...document.querySelectorAll('section[aria-label="Orchestrator MCP bağlantıları"] label.choice')].find(x=>x.textContent.includes('Claude Code'));return l&&l.querySelector('input')})()`;
  await rail('Orchestrator');
  await clickText('Ayarlar', 'document.querySelector(".ws-tabs")');
  await waitFor(`(()=>{const i=${mcpSwitch};return !!i&&!i.disabled})()`, 'MCP panel lists Claude Code as available');
  await ev(`${mcpSwitch}.click()`);
  await waitFor(`!![...document.querySelectorAll('.plan-changes li')].find(l=>l.textContent.includes('.claude.json')&&l.textContent.includes('Eklenir'))`, 'MCP plan shows exactly the .claude.json entry to add', 90000);
  check('MCP planning wrote nothing', readFileSync(join(dirs.home, '.claude.json'), 'utf8') === foreign);
  await shot('10-mcp-plan');
  await clickText('Onayla ve uygula', `document.querySelector('section[aria-label="Orchestrator MCP bağlantıları"]')`);
  await waitFor(`document.body.innerText.includes('Değişiklik uygulandı')`, 'MCP apply confirmed', 120000);
  const withMcp = readJson(join(dirs.home, '.claude.json'));
  const entry = withMcp.mcpServers?.['respected-orchestrator'];
  check('MCP entry written, foreign entry and theme preserved', !!entry && !!withMcp.mcpServers?.['someone-elses'] && withMcp.theme === 'dark');
  check('MCP launcher exists in the managed folder', !!entry && existsSync(entry.args.at(-1)) && entry.args.at(-1).startsWith(dirs.managed));
  const hello = spawnSync(entry.command, entry.args, { input: '{"jsonrpc":"2.0","id":1,"method":"tools/list"}\n', encoding: 'utf8', env: { ...process.env, PATH: pathWithoutPython }, timeout: 60000 });
  check('registered launcher serves tools with no Hub involvement and no system Python', /orchestrator_task_start/.test(hello.stdout ?? ''), (hello.stderr ?? '').slice(0, 200));
  await shot('11-mcp-applied');
  await ev(`${mcpSwitch}.click()`);
  await waitFor(`!![...document.querySelectorAll('.plan-changes li')].find(l=>l.textContent.includes('Kaldırılır'))`, 'MCP removal plan shown', 90000);
  await clickText('Onayla ve uygula', `document.querySelector('section[aria-label="Orchestrator MCP bağlantıları"]')`);
  await waitFor(`document.body.innerText.includes('Değişiklik uygulandı')`, 'MCP removal confirmed', 120000);
  const afterMcp = readJson(join(dirs.home, '.claude.json'));
  check('MCP entry removed again, foreign entry intact', !afterMcp.mcpServers?.['respected-orchestrator'] && !!afterMcp.mcpServers?.['someone-elses']);
  }
} catch (error) {
  check('driver completed without error', false, String(error));
  writeFileSync(join(reportDir, 'page-events.txt'), events.join('\n'));
  try { await shot('failure'); writeFileSync(join(reportDir, 'failure.txt'), await ev('document.body.innerText')); } catch { /* best effort */ }
} finally {
  try { ws?.close(); } catch { /* ignore */ }
  child.kill();
  const guard = spawnSync('python', ['-B', join(tools, 'real_env_guard.py'), 'verify', guardFile], { encoding: 'utf8' });
  check('real AI client configuration untouched by this run', guard.status === 0, (guard.stdout ?? '').trim().slice(0, 300));
  spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore' });
  await sleep(2000);
  rmSync(work, { recursive: true, force: true });
  writeFileSync(join(reportDir, 'page-events.txt'), events.join('\n'));
  const failed = results.filter(r => !r.ok);
  writeFileSync(join(reportDir, 'result.md'), `# Hub native e2e (${new Date().toISOString()})\n\nexe: ${exe}${remote ? ' (remote catalog)' : ''}\n\n${results.map(r => `- ${r.ok ? 'PASS' : 'FAIL'} ${r.name}${r.detail ? ' — ' + r.detail : ''}`).join('\n')}\n\n${failed.length ? `${failed.length} FAILED` : 'ALL PASSED'}\n`);
  console.log(failed.length ? `${failed.length} FAILED` : 'ALL PASSED');
  process.exit(failed.length ? 1 : 0);
}
