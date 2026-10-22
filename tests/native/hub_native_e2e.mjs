// Real desktop end-to-end drive of a built Hub (dev build) through WebView2's DevTools port, on the installer model
// (docs/PRODUCT_CONTRACT.md): the Hub installs, repairs and removes each product by running the
// product's OWN signed installer.
//   node tests/native/hub_native_e2e.mjs <hub.exe> [--remote-catalog] [--intent=brain|orchestrator] [--real-semantic [--seed-semantic DIR]] [--report DIR]
// Safety: the Hub hands installers and products a FAKE user home, and Windows derives the per-user folders from it, so the
// programs, their data and every AI-client configuration land inside that disposable home. Only the Installed-apps entries are
// real (HKCU), so the run refuses to start when one of them exists and removes its own at the end. The real AI-client
// configuration is compared before and after (real_env_guard). The Hub runs with Python removed from PATH, so passing proves
// the products need no system Python.
import { spawn, spawnSync } from 'node:child_process';
import { cpSync, readdirSync, mkdtempSync, mkdirSync, writeFileSync, existsSync, readFileSync, rmSync, statSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, dirname, delimiter } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const args = process.argv.slice(2);
const exe = resolve(args.find(a => !a.startsWith('--')) ?? '');
const remote = args.includes('--remote-catalog');
const hub = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const eco = resolve(hub, '..');
const tools = join(hub, 'release', 'tools');
const reportDir = resolve(args.includes('--report') ? args[args.indexOf('--report') + 1] : join(eco, '.build-cache', 'hub-native'));
mkdirSync(reportDir, { recursive: true });
if (!existsSync(exe)) { console.error('exe not found:', exe); process.exit(2); }

const registered = key => spawnSync('reg', ['query', `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${key}`], { stdio: 'ignore' }).status === 0;
const keys = { brain: '{2F6A9C41-85D7-4B3E-9A10-C7E4D58B1F23}_is1', orch: '{8E2B4D7A-61C3-4F09-A5D8-3B9E17C2F640}_is1' };
const intent = args.find(a => a.startsWith('--intent='))?.slice(9); // product installers open the Hub with --install=<product>
if (!intent && Object.values(keys).some(registered)) {
  console.error('A Respected product is installed on this machine (Installed apps); its installer entry would be replaced, so this run does not start.');
  process.exit(2);
}

const work = mkdtempSync(join(tmpdir(), 'hub-native-'));
const dirs = { catalog: join(work, 'catalog'), managed: join(work, 'managed'), home: join(work, 'home'), vaults: join(work, 'vaults') };
// where the installers put the products under the fake home (its AppData\Local), and where the Hub is told to look
const respected = join(dirs.home, 'AppData', 'Local', 'Respected');
const product = {
  brain: { folder: join(respected, 'RespectedBrain'), exe: join(respected, 'RespectedBrain', 'app', 'respected-brain.exe'), key: keys.brain },
  orch: { folder: join(respected, 'RespectedOrchestrator'), exe: join(respected, 'RespectedOrchestrator', 'app', 'respected-orchestrator.exe'), key: keys.orch },
};
Object.values(dirs).forEach(d => mkdirSync(d, { recursive: true }));
mkdirSync(join(dirs.home, '.gemini'), { recursive: true });
const foreign = '{"mcpServers":{"someone-elses":{"command":"x","args":[]}},"theme":"dark"}';
writeFileSync(join(dirs.home, '.claude.json'), foreign);
const vault = join(dirs.vaults, 'Brain Vault');

// Guard: the run must not change the REAL AI-client configuration (release/tools/acceptance/real_env_guard.py compares before and after).
const guardFile = join(work, 'real-env-before.json');
spawnSync('python', ['-B', join(tools, 'acceptance', 'real_env_guard.py'), 'snapshot', guardFile], { encoding: 'utf8' });
const py = (...a) => { const r = spawnSync('python', ['-B', ...a], { encoding: 'utf8' }); if (r.status !== 0) { console.error(r.stderr); process.exit(2); } return r.stdout.trim(); };
// both products as their own signed installers, exactly as a release ships them (built once, then reused)
if (!intent) {
  py(join(tools, 'build', 'build_bundle.py'), 'orchestrator', '--out', dirs.catalog);
  py(join(tools, 'build', 'build_bundle.py'), 'brain', '--out', dirs.catalog);
}
const served = join(work, 'served');
if (remote) py(join(tools, 'build', 'publish_release.py'), 'local-catalog', ...readdirSync(dirs.catalog).flatMap(name => ['--bundle', join(dirs.catalog, name)]), '--out', served);

const port = 9333;
const pathWithoutPython = process.env.PATH.split(delimiter).filter(p => !/python|WindowsApps/i.test(p)).join(delimiter);
const env = { ...process.env, PATH: pathWithoutPython, RESPECTED_HUB_TEST_HOME: dirs.home,
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`, WEBVIEW2_USER_DATA_FOLDER: join(work, 'webview'),
  RESPECTED_HUB_MANAGED_ROOT: dirs.managed, RESPECTED_HUB_PRODUCTS_ROOT: respected, PYTHONDONTWRITEBYTECODE: '1' };
// development builds trust the local dev signing key only when told where it is (an installed dev build has dev-trust.pub beside it)
env.RESPECTED_HUB_DEV_TRUST = join(eco, '.build-cache', 'signing', 'dev-ed25519.pub');
if (remote) env.RESPECTED_HUB_CATALOG_URL = pathToFileURL(join(served, 'catalog.json')).href; // cache: <managed>/catalog
else env.RESPECTED_HUB_CATALOG_ROOT = dirs.catalog;
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
const labelled = (scope, text) => `[...${scope}.querySelectorAll('label')].find(l=>l.textContent.includes(${JSON.stringify(text)})).querySelector('input,select')`;
const rail = label => clickText(label, 'document.querySelector(".rail")');
const tab = label => clickText(label, 'document.querySelector(".ws-tabs")');
const settingsTab = label => clickText(label, 'document.querySelector(".tabs")');
const refresh = () => ev(`document.querySelector('button[aria-label="Durumu yenile"]').click()`);
const readJson = path => JSON.parse(readFileSync(path, 'utf8'));
const homeEnv = { USERPROFILE: dirs.home, HOME: dirs.home, LOCALAPPDATA: join(dirs.home, 'AppData', 'Local'), APPDATA: join(dirs.home, 'AppData', 'Roaming') };
const cli = (exe, ...a) => { const r = spawnSync(exe, a, { encoding: 'utf8', env: { ...process.env, ...homeEnv } }); try { return JSON.parse(r.stdout); } catch { return { raw: r.stdout + r.stderr }; } };
const walk = dir => readdirSync(dir, { withFileTypes: true }).flatMap(e => e.isDirectory() ? (e.name.startsWith('.') ? [] : walk(join(dir, e.name))) : [join(dir, e.name)]);
// a recorded program file that is plain data, so changing it breaks the product's own check but not the program
const dataFileOf = app => { const files = Object.keys(readJson(join(app, 'distribution.json')).files); return join(app, files.find(f => /\.(txt|md|json)$/.test(f) && f !== 'distribution.json') ?? files.at(-1)); };

try {
  await connect((await target()).webSocketDebuggerUrl);
  await send('Page.enable'); await send('Runtime.enable');
  await waitFor(`document.body.innerText.includes('Respected')`, 'app window renders');
  if (intent) { // a product installer's "Hub ile devam et": the Hub must open on that product's install page, then stop.
    await waitFor(`document.querySelector('.crumbs')?.textContent.includes(${JSON.stringify(intent === 'brain' ? 'Kurulum' : 'Kütüphane')})`, `${intent} hand-over opens its install page`, 30000);
    await shot('intent-' + intent);
    throw new Error('done'); // the intent run ends here; `finally` reports
  }
  check('running as the native desktop app', await ev(`'__TAURI_INTERNALS__' in window`));
  check('Python is not on the PATH of this Hub', !spawnSync('python', ['--version'], { env, encoding: 'utf8' }).stdout);

  await rail('Kütüphane');
  await waitFor(`document.querySelector('.page-head h1')?.textContent==='Kütüphane'&&document.querySelectorAll('article.lib-item').length===2`, 'My Apps lists both products');
  await waitFor(`${cardHas('Brain', 'Kurulu değil')}&&${cardHas('Orchestrator', 'Kurulu değil')}`, 'both products start not installed');
  await shot('01-my-apps-before');

  // Orchestrator through its own installer: the plan names the installer, the product answers afterwards.
  await inCard('Orchestrator', 'Kur');
  await waitFor(cardHas('Orchestrator', 'imzalı kurucusunu'), 'Orchestrator plan names its signed installer');
  await shot('02-plan-orchestrator');
  await inCard('Orchestrator', 'Onayla ve uygula', false);
  await waitFor(cardHas('Orchestrator', 'Kurulu ve doğrulandı'), 'Orchestrator installed and verified', 300000);
  check('Orchestrator is in its own folder with its own Installed-apps entry', existsSync(product.orch.exe) && registered(product.orch.key));
  check('the Hub keeps no copy of the product', !existsSync(join(dirs.managed, 'orchestrator')) && !existsSync(join(dirs.managed, 'versions')));
  await waitFor(`(()=>{const a=${card('Orchestrator')};return !!a&&!a.textContent.includes('Ürüne ulaşılamadı')&&/v0\\.1\\.0/.test(a.textContent)})()`, 'Orchestrator answers over RPP from its installed program', 60000);
  await shot('03-orchestrator-installed');

  // Brain: the wizard collects identity, vault and connections; its installer gets them as /VAULT= and /SET=.
  await inCard('Brain', 'Kur');
  await waitFor(`document.querySelector('.wizard .steps li.current')?.textContent.includes('Kimlik')&&document.body.innerText.includes('Brain\\'i kuralım')`, 'install wizard opens at Identity');
  check('wizard steps are Identity, Memory, Sources, Permissions, Finish', await ev(`[...document.querySelectorAll('.wizard .steps li')].map(l=>l.textContent.replace(/^\\d/,'')).join('|')`) === 'Kimlik|Hafıza|Kaynaklar|İzinler|Onay');
  await setInput(`document.querySelector('.wizard input[placeholder="ör. Ayşe"]')`, 'Test');
  await setInput(`document.querySelector('.wizard input[placeholder="ör. Jarvis"]')`, 'Jarvis');
  await clickText('Devam');
  await setInput(labelled(`document.querySelector('.vault-create')`, 'Klasör'), dirs.vaults);
  await setInput(labelled(`document.querySelector('.vault-create')`, 'Kasa klasörünün adı'), 'Brain Vault');
  await ev(`document.querySelectorAll('input[name=capture]')[1].click()`);  // manual capture
  await clickText('Devam');
  const hooks = `document.querySelector('[aria-label="Hafıza kaynağı AI araçları"]')`;
  const mcpList = `document.querySelector('[aria-label="MCP erişimi verilecek AI araçları"]')`;
  await waitFor(`!!${hooks}&&!!${hooks}.querySelector('[data-client="claude"] input')`, 'Sources step lists the AI clients');
  await ev(`${hooks}.querySelector('[data-client="claude"] input').click()`);
  await clickText('Devam');
  await waitFor(`!!${mcpList}`, 'Permissions step shows per-client MCP switches');
  await ev(`${mcpList}.querySelector('[data-client="claude"] input').click()`);
  await clickText('Devam');
  await waitFor(`document.body.innerText.includes('imzalı kurucusunu')&&[...document.querySelectorAll('button')].some(b=>b.textContent.includes('Onayla ve kur')&&!b.disabled)`, 'approval step shows the installer plan', 120000);
  check('nothing was written before approval', readFileSync(join(dirs.home, '.claude.json'), 'utf8') === foreign && !existsSync(vault) && !existsSync(product.brain.folder));
  await shot('04-approval');
  await clickText('Onayla ve kur');
  await waitFor(`document.body.innerText.includes('Brain ayarları')&&document.body.innerText.includes('Brain Vault')`, 'Brain installed; its settings page follows', 420000);
  await shot('05-brain-installed');
  check('Brain is in its own folder with its own Installed-apps entry', existsSync(product.brain.exe) && registered(product.brain.key));
  check('vault created where the person chose, with the template', existsSync(join(vault, '.respected.json')));
  const settings = cli(product.brain.exe, 'config');
  check('the wizard answers arrived as the product\'s own settings', settings.user_name === 'Test' && settings.assistant_name === 'Jarvis' && settings.capture_mode === 'manual'
    && settings['integrations.clients'] === 'claude' && settings['integrations.mcp_clients'] === 'claude', JSON.stringify(settings).slice(0, 300));
  const claude = readJson(join(dirs.home, '.claude.json'));
  check('foreign MCP entry preserved, Respected entry added in the fake home', !!claude.mcpServers?.['someone-elses'] && !!claude.mcpServers?.['respected-vault'] && claude.theme === 'dark');
  check('only the chosen client was configured (Claude hooks yes, Gemini no)', existsSync(join(dirs.home, '.claude', 'settings.json')) && !existsSync(join(dirs.home, '.gemini', 'settings.json')));
  check('assistant name was written into the vault template', walk(vault).filter(f => f.endsWith('.md')).some(f => readFileSync(f, 'utf8').includes('Jarvis')));

  // Settings page: a schema field changes through the product, and the connection panel switches the MCP entry off.
  await setInput(labelled(`document.querySelector('.panel-glass')`, 'Sabah özeti saati'), '07:30');
  await clickText('Değişiklikleri kaydet');
  await waitFor(`document.body.innerText.includes('geri okunarak doğrulandı')`, 'a Brain setting is saved and read back', 120000);
  check('the product holds the new value', cli(product.brain.exe, 'config', 'get', 'briefing_time').briefing_time === '07:30');
  await waitFor(`${hooks}?.querySelector('[data-client="claude"] input').checked&&!${hooks}.querySelector('[data-client="gemini"] input').checked`, 'connection panel shows the stored per-client choice', 90000);
  await ev(`${mcpList}.querySelector('[data-client="claude"] input').click()`);
  await waitFor(`!!document.querySelector('.plan-changes')`, 'switch-off preview listed', 90000);
  await clickText('Bağlantıları uygula');
  await waitFor(`document.body.innerText.includes('Bağlantılar Brain tarafından uygulandı')`, 'connection change applied by the product', 300000);
  const after = readJson(join(dirs.home, '.claude.json'));
  check('MCP entry removed, foreign entry intact', !after.mcpServers?.['respected-vault'] && !!after.mcpServers?.['someone-elses']);
  await shot('06-brain-settings');

  // Memory page: quick capture -> search finds it -> reader opens it; the note is a new file in the Dump inbox.
  await rail('Brain'); await tab('Hafıza');
  await waitFor(`!!document.querySelector('[aria-label="Hızlı not yakala"]')`, 'Memory page shows search, capture and briefing sections', 60000);
  await ev(`(()=>{const el=document.querySelector('[aria-label="Not içeriği"]');Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(el,'NATIVE-MARMALADE-77 ilk kahvaltı notu');el.dispatchEvent(new Event('input',{bubbles:true}));return true})()`);
  await clickText('Kaydet', `document.querySelector('[aria-label="Hızlı not yakala"]')`);
  await waitFor(`document.querySelector('[aria-label="Hızlı not yakala"]').textContent.includes('Kaydedildi')`, 'quick capture reports the saved note', 90000);
  const dump = join(vault, '📥 000-Inbox', 'Dump');
  check('quick capture created exactly one new note in the Dump inbox', existsSync(dump) && readdirSync(dump).filter(f => f.endsWith('.md')).length === 1);
  await setInput(`document.querySelector('[aria-label="Arama sorgusu"]')`, 'NATIVE-MARMALADE-77');
  await clickText('Ara', `document.querySelector('[aria-label="Not ara"]')`);
  await waitFor(`document.querySelector('[aria-label="Not ara"]').textContent.includes('Dump')`, 'search finds the captured note', 90000);
  await clickText('Oku', `document.querySelector('[aria-label="Not ara"]')`);
  await waitFor(`document.querySelector('[aria-label="Not oku"] pre')?.textContent.includes('ilk kahvaltı notu')`, 'reader shows the note content', 60000);
  await shot('07-memory-page');

  if (args.includes('--real-semantic')) { // optional: runtime + model download into the product's own data folder, then off again
    const sem = `document.querySelector('[aria-label="Anlamsal arama (isteğe bağlı)"]')`;
    const runtime = join(product.brain.folder, 'data', 'semantic-runtime');
    await rail('Brain'); await tab('Ayarlar');
    await waitFor(`!!${sem}&&${sem}.textContent.includes('Kapalı')`, 'semantic panel shows Off (FTS stays the default)', 90000);
    await clickText('Etkinleştir', sem);
    await waitFor(`${sem}.textContent.includes('460 MB')&&${sem}.textContent.includes('Notlarınız bilgisayarınızdan çıkmaz')`, 'semantic confirmation explains size and privacy');
    await clickText('Vazgeç', sem);
    check('declining downloaded nothing', !existsSync(runtime));
    const seed = args.includes('--seed-semantic') ? resolve(args[args.indexOf('--seed-semantic') + 1]) : null;
    if (seed) cpSync(seed, runtime, { recursive: true }); // a verified earlier download, so the run needs no network
    await clickText('Etkinleştir', sem);
    await clickText('Onayla, indir ve etkinleştir', sem);
    if (await waitFor(`${sem}.textContent.includes('Hazır')`, 'semantic panel reaches Ready after download + vector build', 1500000, true)) {
      await clickText('Kapat', sem);
      await waitFor(`${sem}.textContent.includes('Kapalı')`, 'semantic panel switched off again');
    }
  }

  // Orchestrator: a schema setting, the Brain memory link and its MCP registration, all through the product.
  await rail('Orchestrator'); await tab('Ayarlar');
  await waitFor(`document.querySelector('details.settings-block summary')!==null`, 'Orchestrator settings panel');
  await ev(`document.querySelector('details.settings-block summary').click()`);
  await waitFor(`document.querySelectorAll('.settings-block input[type=number]').length>0`, 'Orchestrator settings load');
  await setInput(`document.querySelector('.settings-block input[type=number]')`, '1234');
  await clickText('Uygula', 'document.querySelector(".settings-block")');
  await waitFor(`document.body.innerText.includes('geri okunarak doğrulandı')`, 'Orchestrator settings applied and read back', 60000);
  check('the Orchestrator holds the new value', cli(product.orch.exe, 'config', 'get', 'default_timeout_seconds').default_timeout_seconds === 1234);
  const memory = `document.querySelector('[aria-label="Brain hafıza erişimi"]')`;
  await waitFor(`!!${memory}&&[...${memory}.querySelectorAll('button')].some(b=>b.textContent.includes('Erişimi aç'))`, 'memory access offers to connect');
  await clickText('Erişimi aç', memory);
  await clickText('Onayla ve aç', memory);
  await waitFor(`${memory}.textContent.includes('Açık')`, 'Orchestrator connected to Brain memory', 120000);
  check('the Orchestrator itself reports the link', cli(product.orch.exe, 'memory', 'status').connected === true);
  await clickText('Erişimi kapat ve iptal et', memory);
  await waitFor(`${memory}.textContent.includes('Kapalı')`, 'memory link closed and revoked', 120000);
  check('the link is gone on both sides', cli(product.orch.exe, 'memory', 'status').connected === false);
  await shot('08-orchestrator-settings');
  const mcpPanel = `document.querySelector('section[aria-label="Orchestrator MCP bağlantıları"]')`;
  const mcpSwitch = `(()=>{const l=[...${mcpPanel}.querySelectorAll('label.choice')].find(x=>x.textContent.includes('Claude Code'));return l&&l.querySelector('input')})()`;
  await waitFor(`(()=>{const i=${mcpSwitch};return !!i&&!i.disabled})()`, 'MCP panel lists Claude Code as available');
  await ev(`${mcpSwitch}.click()`);
  await waitFor(`!![...document.querySelectorAll('.plan-changes li')].find(l=>l.textContent.includes('.claude.json')&&l.textContent.includes('Eklenir'))`, 'MCP plan shows exactly the .claude.json entry to add', 90000);
  check('MCP planning wrote nothing', readFileSync(join(dirs.home, '.claude.json'), 'utf8') === readFileSync(join(dirs.home, '.claude.json'), 'utf8') && !readJson(join(dirs.home, '.claude.json')).mcpServers?.['respected-orchestrator']);
  await clickText('Onayla ve uygula', mcpPanel);
  await waitFor(`document.body.innerText.includes('Değişiklik uygulandı')`, 'MCP apply confirmed', 120000);
  const entry = readJson(join(dirs.home, '.claude.json')).mcpServers?.['respected-orchestrator'];
  check('MCP entry names the installed program itself, nothing of the Hub', !!entry && entry.command.toLowerCase() === product.orch.exe.toLowerCase() && JSON.stringify(entry.args) === '["mcp"]', JSON.stringify(entry));
  const hello = spawnSync(entry.command, entry.args, { input: '{"jsonrpc":"2.0","id":1,"method":"tools/list"}\n', encoding: 'utf8', env: { ...process.env, ...homeEnv, PATH: pathWithoutPython }, timeout: 60000 });
  check('registered server serves tools with no Hub involvement and no system Python', /orchestrator_task_start/.test(hello.stdout ?? ''), (hello.stderr ?? '').slice(0, 200));
  await ev(`${mcpSwitch}.click()`);
  await waitFor(`!![...document.querySelectorAll('.plan-changes li')].find(l=>l.textContent.includes('Kaldırılır'))`, 'MCP removal plan shown', 90000);
  await clickText('Onayla ve uygula', mcpPanel);
  await waitFor(`document.body.innerText.includes('Değişiklik uygulandı')`, 'MCP removal confirmed', 120000);
  check('MCP entry removed again, foreign entry intact', !readJson(join(dirs.home, '.claude.json')).mcpServers?.['respected-orchestrator'] && !!readJson(join(dirs.home, '.claude.json')).mcpServers?.['someone-elses']);

  // Repair: a changed program file is reported by the product; the Hub runs the installer again, which restores it.
  await rail('Kütüphane');
  for (const [name, p] of [['Brain', product.brain], ['Orchestrator', product.orch]]) {
    const file = dataFileOf(dirname(p.exe));
    const original = readFileSync(file);
    writeFileSync(file, Buffer.concat([original, Buffer.from('\n')]));
    await refresh();
    await waitFor(cardHas(name, 'Onarım gerekli'), `${name}: the product reports the changed file`, 60000);
    await inCard(name, 'Onar');
    await waitFor(cardHas(name, 'Onayla ve uygula'), `${name}: repair plan shown`);
    await inCard(name, 'Onayla ve uygula', false);
    await waitFor(cardHas(name, 'Kurulu ve doğrulandı'), `${name}: repaired by its installer`, 420000);
    check(`${name}: the changed file is restored`, Buffer.compare(readFileSync(file), original) === 0);
  }
  check('repair kept the settings', cli(product.brain.exe, 'config', 'get', 'briefing_time').briefing_time === '07:30');
  await shot('09-repaired');

  // Removal through each product's own uninstaller: Brain keeps its settings, the Orchestrator is purged.
  writeFileSync(join(dirs.home, '.claude.json'), foreign); // the person's own file, compared byte for byte below
  await inCard('Brain', 'Kaldır');
  await waitFor(cardHas('Brain', 'Ayarlar ve çalışma verileri de silinsin'), 'removal asks about settings and data');
  await inCard('Brain', 'Onayla ve uygula', false);
  await waitFor(cardHas('Brain', 'Kurulu değil'), 'Brain removed', 420000);
  check('program and Installed-apps entry gone, settings and vault kept, AI configs restored', !existsSync(product.brain.exe) && !registered(product.brain.key)
    && existsSync(join(product.brain.folder, 'data', 'config.json')) && existsSync(join(vault, '.respected.json'))
    && readFileSync(join(dirs.home, '.claude.json'), 'utf8') === foreign && !existsSync(join(dirs.home, '.claude', 'settings.json')));
  await inCard('Orchestrator', 'Kaldır');
  await waitFor(cardHas('Orchestrator', 'Ayarlar ve çalışma verileri de silinsin'), 'Orchestrator removal asks about data');
  await ev(`(${card('Orchestrator')}).querySelector('.panel-glass input[type=checkbox]').click()`);
  await inCard('Orchestrator', 'Onayla ve uygula', false);
  await waitFor(cardHas('Orchestrator', 'Kurulu değil'), 'Orchestrator removed', 420000);
  await sleep(3000); // the uninstaller's own copy finishes a moment later
  check('purged: the whole Orchestrator folder and its Installed-apps entry are gone', !existsSync(product.orch.folder) && !registered(product.orch.key));
  await rail('Ayarlar'); await settingsTab('Etkinlik');
  await waitFor(`document.body.innerText.includes('Kurulum işlemleri')&&document.body.innerText.includes('kaldırıldı')&&document.body.innerText.includes('onarıldı')&&document.body.innerText.includes('kuruldu')`, 'Activity lists the Hub\'s own operations');
  await shot('10-activity');
} catch (error) {
  if (String(error) !== 'Error: done') {
    check('driver completed without error', false, String(error));
    writeFileSync(join(reportDir, 'page-events.txt'), events.join('\n'));
    try { await shot('failure'); writeFileSync(join(reportDir, 'failure.txt'), await ev('document.body.innerText')); } catch { /* best effort */ }
  }
} finally {
  try { ws?.close(); } catch { /* ignore */ }
  child.kill();
  spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { stdio: 'ignore' });
  if (!intent) { // leave the machine as it was: remove whatever this run installed (it refused to start over an existing install)
    for (const p of Object.values(product)) {
      const uninstaller = join(dirname(p.exe), 'unins000.exe');
      if (existsSync(uninstaller)) spawnSync(uninstaller, ['/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/PURGE=1'], { stdio: 'ignore', env: { ...process.env, ...homeEnv } });
    }
    await sleep(5000);
    for (const p of Object.values(product)) { // an entry whose program went with the fake home: remove it, it is this run's own
      rmSync(p.folder, { recursive: true, force: true });
      if (registered(p.key)) spawnSync('reg', ['delete', `HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\${p.key}`, '/f'], { stdio: 'ignore' });
    }
    check('nothing of this run is left installed', Object.values(product).every(p => !existsSync(p.folder) && !registered(p.key)));
  }
  const guard = spawnSync('python', ['-B', join(tools, 'acceptance', 'real_env_guard.py'), 'verify', guardFile], { encoding: 'utf8' });
  check('real AI client configuration untouched by this run', guard.status === 0, (guard.stdout ?? '').trim().slice(0, 300));
  await sleep(2000);
  rmSync(work, { recursive: true, force: true });
  writeFileSync(join(reportDir, 'page-events.txt'), events.join('\n'));
  const failed = results.filter(r => !r.ok);
  writeFileSync(join(reportDir, 'result.md'), `# Hub native e2e (${new Date().toISOString()})\n\nexe: ${exe}${remote ? ' (remote catalog)' : ''}${intent ? ` (intent ${intent})` : ''}\n\n${results.map(r => `- ${r.ok ? 'PASS' : 'FAIL'} ${r.name}${r.detail ? ' — ' + r.detail : ''}`).join('\n')}\n\n${failed.length ? `${failed.length} FAILED` : 'ALL PASSED'}\n`);
  console.log(failed.length ? `${failed.length} FAILED` : 'ALL PASSED');
  process.exit(failed.length ? 1 : 0);
}
