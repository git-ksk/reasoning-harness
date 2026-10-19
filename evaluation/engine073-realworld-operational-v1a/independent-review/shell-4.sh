set -euo pipefail
if test -d /tmp/results; then
  sudo chown -R "$(id -u):$(id -g)" /tmp/results
fi
