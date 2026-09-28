# Engine 0.6 evidence relevance independent holdout v1

Status: pre-freeze / unobserved。holdout freeze tag / live observationはまだ存在しない。

## Independence boundary

このholdout corpusをauthorする前に、calibration v23はcommit `0871169303d20c464b0454dbe0a150d2fef0ec43` のfirst/only canonical run `36400085595` でPASS済み。

Holdout identity:

- suite: `evidence-relevance-holdout-v1`
- issue: #462
- cases: 26
- corpus: `fixtures/evidence-relevance-holdout-v1/manifest.json`
- planned freeze tag: `engine-0.6-evidence-relevance-holdout-v1-freeze`
- workflow: `.github/workflows/engine-0.6-evidence-relevance-holdout-v1-live.yml`
- seed: `4629101`
- required provider: Mistral `ministral-8b-latest`、Groq `openai/gpt-oss-120b`、Google `gemini-3.5-flash-lite`
- annotation protocol: `evidence-relevance-effective-qualification-v23`
- materialization: v16

holdoutは新規ID / task / target question / canonical entity name / signal textだけで構成する。48-case v23 calibration corpusとのexact reuseはlive observation前のholdout test surfaceで拒否する。

## Coverage

独立authorした26件は Relevant 8 / Irrelevant 10 / Ambiguous 8。

- exact target availability;
- Harness-authorized alias;
- semantic-equivalent positive relevance;
- title/bodyへ分散したchange evidence;
- 日本語task + 英語evidence;
- structured metadata identity;
- multi-section benefit evidence;
- target名をURL slugに含まないrelevant evidence;
- same target / wrong relation;
- sibling target / same relation;
- navigationのみのtarget mention;
- target-local supportのないgeneric landing;
- unrelated announcement;
- comparison-only target mention;
- relevance自己宣言を試みるprompt injection;
- explicitly distinct structured sibling;
- sibling target + different relationのv23 composition boundary;
- possible rename / alias / successor ambiguity;
- partial / multi-product identity ambiguity;
- truncated relation context;
- URL-only identity;
- shared-row ownership ambiguity。

production motivating contentは含めない。

## Frozen semantics and gates

holdoutでproduction semantic ruleは追加しない。calibration v23でacceptedになったeffective qualification v3 + materialization v16をそのまま実行する。

3 provider全てrequired。PASSには各arm 26/26 operational completeに加え以下を要求する。

- wrong-target relevance retention = 0;
- false relevance rejection = 0;
- Relevant left Ambiguous = 0;
- utility miss = 0;
- materialized exact = 26/26;
- authority-qualified effective qualification = 26/26;
- effective qualification risk miss/spurious risk = 0;
- authority identity/relation miss = 0。

provider failureはsemantic failureと分離して記録するが、required armはincomplete扱い。holdoutはfirst/onlyでrerun拒否。FAILならv1をtuning/rescoreせず、新successor identityが必要。

## Groq admission plan

calibration v23のGroq実測は120,075 tokens。直後のTPD headroomを再利用しないため、holdoutは保守的lower-bound modelを使う。v23開始headroomはconfigured 105Kのみと仮定し、frozen minimum pacing refillだけを加算する。v23がfloorより遅く開始した分とmodel execution中のrefillは安全側に無視する。

この前提では `2026-09-28T11:50:54Z` のv23終了時modeled headroomは約8.9K tokens。

holdout v1:

- modeled Groq start headroom: 55K;
- earliest modeled floor: `2026-09-28T17:22:57Z`（JST `2026-09-29 02:22:57`）;
- 26 cases;
- case間delay: 300秒;
- provider request spacing: 10秒;
- minimum deliberate pacing: 7,760秒;
- pacing中modeled refill: 約17.96K tokens;
- modeled supply: 約72.96K tokens;
- observed-token hard cap: 70K;
- pre-case reserve: 4K;
- v23 case当たり実測からのprojected demand: 約65.0K tokens。

materialなorganization-level Groq追加利用が既知または疑わしい場合、このmodelは無効としてfreeze前にdelay / re-anchorする。freeze直前のfresh synthetic readinessも必須だが、transport / credential / TPM / RPD evidenceに限定しTPD-headroom proofとは扱わない。

## Freeze rule

first/only holdout observation前に:

1. exact candidate clean + standard CI green;
2. holdout checksum PASS;
3. validate-only 26 planned / 0 observed;
4. calibration content exact-reuse check PASS;
5. v23 regression + holdout-runner test PASS;
6. modeled Groq floor経過;
7. exact candidateでfresh Groq readiness PASS;
8. annotated holdout freeze tagを1回だけ作成。

freeze後はcorpus text / expected label / scorer / threshold / provider role / prompt contract / production semanticsをこのholdout identity内で変更しない。
