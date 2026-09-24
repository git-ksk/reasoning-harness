# Engine 0.6 candidate: evidence-need routing calibration v1

Status: fresh calibration prepared。live model observationはまだ記録していない。

Issue #461はacquisition前にtarget-local evidence-need layerを追加する。この文書ではindependent holdout authoring / observationより前にcandidate acceptance ruleをfreezeする。release済みEngine 0.5.0 semanticsはimmutableのまま保持する。

## Candidate boundary

candidateは「このexact targetが要求するepistemic mode」と「そのmodeに対して今新規acquisitionが必要か、既存のvalid evidenceをreuseできるか」を分離する。

modeは no_factual_evidence、context_only、external_optional、external_required、trusted_verification_required。

modelが出せるのはadvisoryなtarget-local proposalだけである。Harness-owned policyがbaseline、minimum floor、explicit downgrade permission、current-state requirement、explicit user verification intent、trusted-verification requirement、context sufficiency、existing-evidence reuse statusを所有する。materializerはrequirementをpreserve/escalateし、proposalによるloweringは明示されたHarness-owned downgrade floor内だけ許可する。

context completenessとcontext sufficiencyは別軸である。partial/truncated contextをabsenceの証拠にしない。exact targetについてHarnessがsufficientと明示した場合だけcontext-localを維持できる。

existing evidence reuseはevidence needと別軸である。external_requiredでも、already-admitted evidenceがfreshness、scope、authority、policy identityを満たせばnew tool callは不要。stale、scope mismatch、policy mismatch、ambiguous evidenceはreuse不可。

## Fresh calibration corpus

Fixture: fixtures/evidence-need-routing-calibration-v1/manifest.json

22件のsynthetic caseで、no-factual transformation、article summary/explanation、field extraction、claims-about-content / claims-about-world、current availability、regional availability、mixed targets、partial/truncated context、conflict、context内prompt injection、follow-up escalation/preservation、external optional、trusted exact scalar verification、valid/stale evidence reuse、resolver unavailable operational boundary、semantic ambiguityのconservative escalationを扱う。

motivating Cloud production incidentはcalibrationおよび後続holdout tuningから除外する。

## Correctness gates

holdout authoring前に次をfreezeする。

- unsafe skipped acquisition = 0
- context authority laundering = 0
- false context-local answer authorization = 0
- explicit Harness-owned downgrade floor外でのmodel weakening = 0
- explicit user verification intent downgrade = 0
- current-state downgrade to context-only = 0
- trusted-verification downgrade = 0
- stale/scope/policy-invalid evidence reuse = 0
- mixed-target whole-turn over-routing = 0
- replayed external side effects = 0

Engine 0.6 candidate全体では、後続#462/#463のwrong-target relevance admission、source-attributed truth promotion、renderer-only unsupported factual exposure、source-binding violation、paraphrase/translation strengtheningもすべて0を要求する。

## Utility metrics

correctnessとutilityは別々にreportする。

- unnecessary acquisition rate
- avoidable abstention rate
- target-local routing accuracy
- mixed-target composition accuracy
- follow-up mode-transition accuracy
- valid existing-evidence reuse rate
- model calls
- external tool calls
- input/output tokens
- latency
- provider/model operational failures

always-external-required candidateは安全という理由だけではutility PASSにしない。always-context-only candidateはcorrectness FAILとする。

## Live calibration runner

実装済みrunner:

```bash
cargo run -p reasoning-harness-cli --bin reason-evidence-need-study -- \
  fixtures/evidence-need-routing-calibration-v1 \
  --provider <provider> \
  --model <model> \
  --seed <seed> \
  --checkpoint /tmp/evidence-need-calibration-checkpoint.json
```

provider callなしのcontract/corpus preflightは `--validate-only` で実行でき、22 caseすべてのpolicy materializationを検証する。

canonical full calibrationは `--fixture` を指定せず22 caseすべてを1回観測する。runnerはproposal exact match、materialized mode、acquisition disposition、correctness-boundary violation、utility miss、provider failure、token/latencyを別軸で記録する。JSON-Schema transportがunsupportedまたはstrict parse failureの場合だけ、既存のbounded JSON-object fallbackを1回使用し、fallback利用とprovider attemptを明示する。

in-progress checkpointはnon-scorableで、provider failureを含むcompleted runもoperationally incompleteとしてscoring対象外にする。raw model responseやcredentialはcheckpoint/outputへ保存しない。

既存research runnerと同様、このrunnerのlive provider credentialはprovider環境変数（`MISTRAL_API_KEY` / `GEMINI_API_KEY` / `GROQ_API_KEY` / `NVIDIA_API_KEY`）から解決する。

現在のローカル `reason auth status` では Mistral / Google / Groq / Nvidia のeffective credential sourceがすべてmissingであり、live observationはまだ実行していない。このoperational prerequisiteはsemantic resultと分離して扱う。

## Evaluation sequence

1. 最初のrecorded live calibration observationまではこのcalibration suiteだけをmutable surfaceとする。
2. model qualityとは独立にdeterministic materialization testsを実行する。
3. `reason-evidence-need-study` でproposal modeとHarness-materialized mode / acquisition dispositionを別々に記録する。runner実装済み。
4. tuningはこのfresh calibration identityだけに対して行う。
5. candidate semantics / thresholdsをfreezeする。
6. acceptance criteria freeze後に別のindependent holdoutをauthorする。
7. holdoutはfirst observation前にfreezeし、observation後のmutation/rescoreを禁止する。
8. operational/provider failureはsemantic scoreと分離する。
9. correctness / utilityの両gateをPASSして初めてEngine version promotionを検討する。

このcalibration準備だけではEngine 0.6 release coordinateを作らない。
