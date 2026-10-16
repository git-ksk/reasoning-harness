"""Offline subprocess only; never imports a provider or accesses a socket."""
import argparse
import json
import os
from pathlib import Path
import signal
import time

p = argparse.ArgumentParser()
p.add_argument('--mode', required=True)
p.add_argument('--contract', default='normal')
p.add_argument('--marker', required=True)
p.add_argument('--provider')
p.add_argument('--model')
a = p.parse_args()
with Path(a.marker).open('a') as f:
    f.write(a.mode + '\n')
    f.flush()
    os.fsync(f.fileno())

def usage(tokens=11):
    return {'tracked_from_session_start': True, 'total_tokens': tokens, 'provider_attempts': 1}

def failure(kind, lower=False):
    return {'result': {'status': 'failed', 'failure': {'failure_class': kind},
        'partial_diagnostics': {'complete': False, 'usage_is_lower_bound': lower,
                                'usage': {'run': usage(7)}}}}

def cancelled(signum, frame):
    print(json.dumps(failure('cancelled', True)), flush=True)
    Path('partial-trace.json').write_text(json.dumps({'cancelled': True}))
    raise SystemExit(1)

if a.mode in ('descendant-on-cancel', 'descendant-on-exit'):
    # A real descendant in the leader's new session, with no sockets/providers.
    child = os.fork()
    if child == 0:
        signal.signal(signal.SIGINT, signal.SIG_IGN)
        Path('descendant.pid').write_text(str(os.getpid()))
        while True:
            with Path('heartbeat.txt').open('a') as stream:
                stream.write('alive\n')
                stream.flush()
                os.fsync(stream.fileno())
            time.sleep(0.01)
    while not Path('heartbeat.txt').exists():
        time.sleep(0.005)
    if a.mode == 'descendant-on-cancel':
        signal.signal(signal.SIGINT, cancelled)
        while True:
            time.sleep(0.02)
if a.mode in ('cooperative', 'hard'):
    signal.signal(signal.SIGINT, cancelled if a.mode == 'cooperative' else signal.SIG_IGN)
    while True:
        time.sleep(0.02)
if a.mode == 'crash':
    os._exit(3)
if a.contract == 'helper':
    obj = {'equivalent': False, 'operational_failure': 'provider_Quota' if a.mode == 'quota' else None,
           'usage': {'total_tokens': 10}, 'provider_attempts': 1}
else:
    obj = failure({'quota':'quota', 'capacity':'timeout', 'protocol':'protocol'}[a.mode]) if a.mode in ('quota', 'capacity', 'protocol') else {
        'result': {'output_contract': 'reason-natural-output-v4', 'usage': {'run': usage()}}}
if a.mode == 'missing':
    obj['result']['usage']['run'].pop('total_tokens')
if a.mode == 'overshoot':
    obj['result']['usage']['run']['total_tokens'] = 100000
print(json.dumps(obj), flush=True)
