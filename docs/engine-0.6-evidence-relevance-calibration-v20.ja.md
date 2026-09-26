# Engine 0.6 evidence-target relevance calibration v20 — design-only successor

Status: design only。v20 runtime実装、workflow、freeze tag、live observationはまだ存在しない。

v20はimmutable v19 run `36247789205` のsuccessor。replacement observationではなく、v19結果の再解釈にも使わない。

## Design objective

required Mistralで48/48に到達したv19 authority splitを維持しつつ、Google replicationで露出した、v19 live前から候補化済みのrelation-negative境界を1つだけ扱う。Groq daily quotaを理由にacceptanceを緩めない。

## Frozen inheritance

変更しないもの:
- fixed core `evidence-relevance-fixed-core-v1`、48件固定;
- 全label / expected disposition;
- primary proposal v5;
- typed deterministic local-risk classifier;
- raw verifier v8 telemetryはdiagnostic-only;
- Harness-owned effective qualificationをruntime/gating authorityとする分離;
- strict positive identity floor;
- v19 target-negative terminal rule;
- provider retry、time budget、attempt telemetry、quota/capacity latch、sanitization;
- required providerはMistral + Groq;
- Googleはfull non-gating replication;
- one-shot canonical immutabilityとPASS前holdout禁止。

## Proposed v20 semantic delta

generic pre-freeze proofが通った場合にのみ採用する。

以下を全て満たす場合:
- deterministic local risk = `none`;
- effective `scope_risk=none`;
- effective `identity_scope=exact_target`;
- effective `relation_scope=different_relation`;
- primary `target_binding=exact`;
- primary `relation_binding != exact`;
- raw model safety riskにnon-noneがない;

AmbiguousではなくIrrelevantをmaterializeする。

primary relationがexact、identity uncertainty、context/ownership/mapping risk、conflicting safety signalのいずれかがあればfail closedしてterminal rejectionしない。

このcandidateを許容する根拠:
- v19 live observation前から明示済み;
- Google v19 case 13が同じdisagreementを実際に示した;
- frozen coreの`exact_target + different_relation + no risk` expectationは3件すべてIrrelevantで、Ambiguous collisionは0;
- positive-admission pathは増えない。

実装時は新しいmaterialization policy IDを与え、v14をhistoricalのまま保持する。

## Context-gap relation normalization: deferred

Google v19ではnon-none `context_gap`下でeffective relation-scope mismatchが3件出たが、最終dispositionは全て正しくAmbiguousだった。qualification gateを緩めたりfixture固有phraseに寄せたりして直さない。

normalizationを検討する前に、clipped textやomitted ownershipを推測せず、`requested_relation`と`unresolved`の両方を含むfrozen context-gap expectation全件を再現できるgeneric deterministic ruleが必要。証明できなければv20ではnon-gating diagnostic disagreementのまま維持する。

## Operational successor policy

Groq v19 FAILはtyped daily quotaでありsemantic missではない。したがってv20では:
- Groq requiredを維持;
- daily quota fail-fast + 即provider-arm latchを維持;
- retry stormやsemantic retryを追加しない;
- manual quota probeをacceptance evidenceにしない;
- v19をrerunしない;
- v20 implementationと全pre-freeze checkがgreenになった後、別quota windowでfresh v20 canonicalを1回だけ実行できる。

Groqが再度quotaに当たれば、そのv20 canonicalもimmutable FAIL。provider-role変更は別の独立証拠を必要とし、#462をPASSさせる目的だけでは変更しない。

## Required pre-freeze proof

v20 freeze tag前に必須:
- case growth / relabel 0;
- relation-negative ruleのgeneric property test。conflicting primaryとsafety-risk counterexampleも含む;
- frozen v19 Mistral replayがeffective qualification 48/48、materialization 48/48を維持;
- frozen v19 Groq successful observation 10件が10/10 exactを維持;
- frozen v19 Google successful observationがmaterialization 45/46から46/46へ改善し、unsafe Relevant / false rejection / Relevant -> Ambiguous regression 0;
- Google context-gap relation mismatch 3件はgenericにregressionなしで解消するか、non-gating diagnostic disagreementとして明示的に維持;
- v18/v19 regression green;
- full workspace test、Clippy `-D warnings`、fmt、surface checksum、validate-only green;
- public artifactにprivate-project identifier / secret / local pathがない。

このdesign-only phaseではv20 live run禁止。

## Holdout boundary

first/only frozen v20 canonical PASSまでindependent holdout authoringは禁止。v19 artifactはsuccessor design evidenceであり、rescoreせずholdoutとしても扱わない。
