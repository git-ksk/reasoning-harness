# CI pinned-baseline fetch correction — #533

[First failing run38587414946](https://github.com/git-ksk/reasoning-harness/actions/runs/38587414946/job/115932987185) at PR#532 head `6afb4d28c78f54f6512d6af51ef75f1fbe383fed` failed before independent-runner tests: `git cat-file -e 4cae9326bcc3e210c3250f05772d84f00e41b645^{commit}` returned128 / `Not a valid object name`.

A clean local shallow repository fetching main `27737ead2b0dfeddcdcebff608e8957bd263c4e0` at depth20 reproduces the missing object. Fetching the same pinned commit directly at depth1 restores the object check; the exact fetch was also executed successfully against GitHub. See `reproduction.json`.

The mutable ordinary CI workflow now fetches the exact frozen commit instead of relying on main remaining within an arbitrary depth. Baseline SHA, cat-file validation, source diff and runner tests remain unchanged. Historical evaluation workflows, fixtures, scores and first observations are not modified. This is a checkout-infrastructure repair, separate from #465 MCP transport and from Engine/provider semantics. Final-head CI must pass before merge; the original failure remains recorded.
