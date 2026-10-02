# Engine 0.6 evidence relevance independent holdout v4

Status: **pre-freeze / unobserved**。

Successor v4 semanticsはcommit 663ade43de96dbda511a11f351e8ce1593d85ed1 / tag engine-0.6-evidence-relevance-successor-v4-semantics-freeze で先にfreeze済み。fresh holdout v4 corpusはそのfreeze後にのみauthorした。

## Frozen semantic baseline

- effective qualification: v6
- materialization: v19
- successor-v4 semantic checksum: fixtures/evidence-relevance-holdout-successor-v4/semantics-v4.sha256
- historical v23 / holdout v1 / v2 / v3a observationはimmutable
- runner routingは v1=v3/v16、v2=v4/v17、v3=v5/v18、v4=v6/v19 を維持

## Fresh surface

- suite: evidence-relevance-holdout-v4
- cases: 26 = Relevant 8 / Irrelevant 10 / Ambiguous 8
- annotation protocol: evidence-relevance-effective-qualification-v6
- fixed core: evidence-relevance-fixed-core-v4
- corpus status: fresh_unobserved_holdout

v4 surfaceは新規case ID / canonical entity / task / candidate textを使用する。

immutable holdout v1/v2/v3に対してcase-ID、canonical entity、task、exact candidate signal、exact 8-token candidate-signal n-gramのoverlapをすべて0とする。v3a terminal case ID/entityとfailed v1/v2 motivating entityも明示的に除外する。

corpusはv3a missのrelabel / paraphrase setではない。exact identity、authorized alias、semantic-equivalent policy、distributed support、relation mismatch、repeated sibling ownership、navigation-only mention、explicit absence、mapping uncertainty、shared ownership、truncation、URL-only identity、single-signal near siblingをfreshなpositive / negative / abstention controlで構成する。

## Groq TPD admission

次canonical Groq armはimmutable v3a actualから保守的に再anchorする。

- v3a configured start headroom: 55,000
- v3a pacing: 300 seconds
- minimum request interval: 10 seconds
- v3a observed tokens: 65,015
- conservative completion anchor: 2026-09-29T17:12:55Z
- modeled post-v3a headroom: 7,947.96
- required v4 start headroom: 55,000
- v4 self-budget: 70,000
- pre-case reserve: 4,000
- v4 pacing: 300 seconds
- fail-closed earliest floor: 2026-09-29T22:51:42Z / 2026-09-30 07:51:42 JST

workflowはfrozen constantからfloorを再計算する。既知のmaterialなorganization-level Groq usageがv3a後に存在する場合、このmodelは無効でfreeze tag push前に再anchorが必要。tiny readinessはtransport / credential / TPM / RPD evidenceに限り、TPD headroom proofではない。

## Pre-freeze validation contract

engine-0.6-evidence-relevance-holdout-v4-freeze 作成前に:

- v4 distribution / frozen expected semantics PASS
- v1/v2/v3 independence audit PASS
- validate-only = 26 planned / 0 observed
- successor-v4 immutable replay full exact維持
- full core/providers/CLI PASS
- workspace all-target Clippy -D warnings PASS
- rustfmt / diff check PASS
- workflow YAML parse PASS
- surface-v4.sha256 exact validation PASS
- production special-case scan clean

最初のv4 freeze tagだけをcanonicalとし、tag移動・再作成・replacement observation目的のrerunは禁止。

## Local pre-freeze validation

- v4 frozen expected semantics: 26/26 PASS
- v1/v2/v3 independence audit: PASS
- exact 8-token candidate-signal overlap: v1/v2/v3に対して0
- v3a terminal-case/entity exclusion: PASS
- validate-only: 26 planned / 0 observed
- successor-v4 replay: v23 48/48 x3、holdout v1 26/26 x3、holdout v2 26/26 x3、holdout v3a 26/26 x3
- focused v4 manifest test: 2/2 PASS
- holdout runner test: 4/4 PASS
- core: 587 passed / 0 failed
- providers: 153 passed / 1 ignored / 0 failed
- CLI: 368 passed / 4 ignored / 0 failed
- workspace all-target Clippy -D warnings: PASS
- rustfmt / diff check: PASS
- workflow YAML: 93/93
- production special-case scan: clean
