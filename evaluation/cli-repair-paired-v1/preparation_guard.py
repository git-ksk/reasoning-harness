"""This preparation snapshot only permits inert numeric-loopback callbacks."""
import json
from pathlib import Path
from urllib.parse import urlsplit


def require_loopback_preparation(model, env):
    protocol = json.loads((Path(__file__).parent / 'protocol.json').read_text())
    if protocol['state'] != 'PREPARATION_NO_LIVE_ALLOWED':
        raise ValueError('new_execution_admission_not_implemented')
    url = urlsplit(env.get('ENGINE073_LOOPBACK_BASE_URL', ''))
    if not (model == 'fixture-model' and url.scheme == 'http'
            and url.hostname == '127.0.0.1' and url.port is not None
            and url.username is None and url.password is None
            and url.path == '/' and not url.query and not url.fragment):
        raise ValueError('paired_preparation_requires_inert_loopback')
    for name in ['MISTRAL_API_KEY', 'GEMINI_API_KEY', 'GOOGLE_API_KEY', 'GROQ_API_KEY', 'NVIDIA_API_KEY']:
        if env.get(name) not in [None, 'inert-loopback-only']:
            raise ValueError('paired_preparation_disallows_real_credentials')
