"""How long one Hub call to the INSTALLED products takes (each call starts the program). Run before and after installing a new build:
   python common/measure-rpp.py
Read-only: it only asks manifest, health and settings.readback."""
import json, os, subprocess, time, tempfile, concurrent.futures as cf

L = os.environ['LOCALAPPDATA']
EXE = {'brain': L + r'\Respected\RespectedBrain\app\respected-brain.exe',
       'orchestrator': L + r'\Respected\RespectedOrchestrator\app\respected-orchestrator.exe'}

def call(product, op, payload=None):
    req = json.dumps({'rpp': '1', 'op': op, 'operation_id': None, 'payload': payload or {}}).encode()
    t = time.perf_counter()
    out = subprocess.run([EXE[product], 'rpp'], input=req, capture_output=True, cwd=tempfile.gettempdir(), creationflags=0x08000000)
    ms = (time.perf_counter() - t) * 1000
    ok = json.loads(out.stdout or b'{}').get('ok')
    return ms, ok

for product, ops in (('brain', ['manifest', 'health', 'settings.readback']), ('orchestrator', ['manifest', 'health'])):
    for op in ops:
        runs = [call(product, op) for _ in range(3)]
        print(f'{product:13} {op:18} ' + ' '.join(f'{ms:5.0f}' for ms, _ in runs) + f' ms  ok={all(ok for _, ok in runs)}')

# what the Hub does on one refresh: manifest+health for two products at once, and what the Brain overview adds
t = time.perf_counter()
with cf.ThreadPoolExecutor(8) as pool:
    jobs = [pool.submit(call, p, o) for p in ('brain', 'orchestrator') for o in ('manifest', 'health')]
    [j.result() for j in jobs]
print(f'4 calls in parallel (one eco refresh): {(time.perf_counter() - t) * 1000:.0f} ms')
t = time.perf_counter()
with cf.ThreadPoolExecutor(8) as pool:
    jobs = [pool.submit(call, 'brain', o) for o in ('settings.readback', 'manifest', 'health')] + [pool.submit(call, 'brain', 'memory.note.read', {'path': 'daily/2026-10-22.md'}) for _ in range(3)]
    [j.result() for j in jobs]
print(f'6 brain calls in parallel (Brain overview): {(time.perf_counter() - t) * 1000:.0f} ms')
