# Action materialization v3 — 採用結果

Issue #283 の3回目の fresh adoption holdout を `action-materialization-v3` として freeze して実行した。このsuccessorでは、Issue #283のaction-selection/materialization境界と、downstream #248 finalizationを明示的に分離して評価した。

両provider coordinateともfreeze済みadoption gateをPASSした。candidateは、測定対象のcorrectness boundaryとsame-key sibling identityを維持しながら、対象pathのlegacy executable-ID planner callを0にし、すべてのcomplete caseでHarness-owned intent materialization pathを通した。

## Frozen coordinate

- freeze tag: `action-materialization-v3-freeze`
- freeze commit: `34eada58093d8e4086849a6d8c3ac08882be8448`
- control product commit: `94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e`
- candidate product commit: `7a91d272af1bab0a97bf80ed7bba027ff253d50a`
- corpus: `action-materialization-v3`
- evaluator: `reason-action-materialization-v3`
- scoring: `action-materialization-scoring-v3`
- canonical Actions run: `35431076755`
- Mistral job: `105865637487`
- Google job: `105865637557`
- primary seeds: `104211`–`104215`
- observed all-k group: `k=5`

surface、scoring policy、provider/model coordinate、case identity、seed、coordinate order、adoption predicateは、初回live provider callより前にfreeze済み。semantic rerunは行っていない。

## 採用結果

| provider/model | control complete | candidate complete | control action-path success | candidate action-path success | correctness violations, control | correctness violations, candidate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` | 5/5 | 5/5 | 5/5 | 5/5 | 0 | 0 |
| Google `gemini-3.5-flash-lite` | 5/5 | 5/5 | 4/5 | 5/5 | 0 | 0 |

両provider jobともfreeze済みacceptance全項目をPASSした。

### Architecture path の実測

両providerで以下を確認した。

- candidateの全10 complete caseで別々のsame-key sibling target identityを保持;
- controlの全10 complete caseでlegacy executable-action planner pathを使用;
- candidateの全10 complete caseで `reason-investigation-intent-v1` + `target-intent-materialization-v1` にconform;
- candidateのlegacy executable-action planner callは0;
- candidateのintent rejection / action rejectionは0;
- relevant read-only capability selectionを維持;
- correctness-boundary violationは0;
- IIDを仮定した `p^k` は算出していない。

path別call数:

| provider/model | control legacy executable-action planner calls | candidate legacy executable-action planner calls | candidate intent calls | candidate Harness materializations |
| --- | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` | 20 | 0 | 30 | 30 |
| Google `gemini-3.5-flash-lite` | 20 | 0 | 20 | 15 |

Google control coordinateでは、seed `104214` のstale Solvane caseでaction-path failureが1件あった。stochastic control plannerが、既にattempt済みの `target-primary` / `solvane-window-secondary` pairを再提案し、Harnessが `duplicate_action` として正しくrejectした。このcaseでもcorrectness-boundary violationは0。candidate coordinateはaction/intent rejectionなしで5/5 action-path trialを完了した。

fresh surface上で、candidateが対象のmechanically safe pathからstochastic executable-ID selectionを取り除きつつ、検証したfail-closed / correctness boundaryを維持していることを観測できた。

## Downstream finalization は別軸

downstream finalizationは記録するが、v3 adoption gateには含めていない。

Mistralでは既知のfinalization interactionが引き続き観測された。

- control: `requires_verification` 9 case、`unresolved` 1 case;
- candidate: `requires_verification` 10 case;
- grounded caseのfalse abstention: control 5、candidate 5。

Googleではcontrol/candidateで同じdownstream outcome分布になった。

- `grounded_answer`: 5 case;
- `requires_verification`: 3 case;
- `unresolved`: 2 case;
- grounded caseのfalse abstention: 0。

v3は#248 finalization semanticsを変更していない。Issue #283のaction-materialization採用判定とdownstream finalization outcomeを混同しないよう評価軸を分離しただけである。

## Cost / latency 観測

各provider 5 paired trialのみの記述的測定で、一般化しない。

Mistral candidate vs control:

- provider calls: 60 → 70 (+16.67%)
- provider attempts: 74 → 88 (+18.92%)
- tokens: 58,139 → 64,735 (+11.35%)
- provider latency: 110,423 ms → 137,787 ms (+24.78%)
- wall time: 111,852 ms → 139,234 ms (+24.48%)

Google candidate vs control:

- provider calls: 55 → 55 (0.00%)
- provider attempts: 59 → 63 (+6.78%)
- tokens: 39,070 → 34,629 (-11.37%)
- provider latency: 164,801 ms → 260,004 ms (+57.77%)
- wall time: 165,533 ms → 260,604 ms (+57.43%)

したがってIssue #283をmodel call総数、token、latencyの最適化として扱わない。採用するarchitecture benefitは、mechanically safeな対象pathでexecutable capability identityをplannerのstochastic selectionから外し、Harness-ownedにすること。

## Deterministic / freeze validation

v3 freeze前:

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo test --workspace`: PASS
- v3 evaluator unit tests: 5/5 PASS
- fixture preflight: PASS
- surface checksum verification: PASS
- YAML parse: PASS
- `git diff --check`: PASS
- candidate commit `7a91d272...` とproduct inputsの差分: 0

canonical両jobでも、credentials/live observationより前にfreeze済みsurfaceとexact product coordinateを再検証している。

## Disposition

fresh v3 evidenceにより、評価対象architectureについてIssue #283 adoption requirementを満たした。

- mechanically safe action materializationはHarness-owned;
- candidateは対象pathからlegacy stochastic executable-ID selectionを除去;
- same-key sibling identityを維持;
- retry / attempted-pair rejectionはfail closedを維持;
- canonical observationのcorrectness-boundary violationは0;
- downstream finalization authorityは別系統のまま変更なし;
- v1/v2/v3を通してcontrol/candidate product commitは固定。

v1 failed measurement、v2 mixed result、v3 adoption resultをすべて履歴として残し、product candidateをPR review/mergeへ進められる。

## 保存したmachine report

- [Mistral raw machine report](observations/action-materialization-v3-mistral-run-35431076755-2026-09-19.json), SHA-256 `01eddaf5d14a1e456b803a4e2a5e5a06fba5f7df9cc4fc6ced444b246d735996`
- [Google raw machine report](observations/action-materialization-v3-google-run-35431076755-2026-09-19.json), SHA-256 `0d0062eea2980a8a4bf231158c336eb4105c16748f736cd51f1eb83c4b6809e8`
