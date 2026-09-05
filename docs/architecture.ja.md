# アーキテクチャ

## プロダクト境界

Reasoning Harness は、確率的な candidate generation を取り囲む native correctness runtime である。現在の core は `ReasoningArtifact` を materialize して評価し、provider-neutral な bounded protocol も担う。この protocol は、未解決の reasoning を resolution request に変換し、採用した evidence や改訂 candidate を再検証し、十分にカバーされた proposition だけから answer を finalize できる。

model は trusted computing base の一部ではない。model は facts、claims、links、transformations、repairs、rendered prose を提案できるが、結果の state が構造上 admissible か、またどの程度の support を主張できるかは harness が決める。

### 現在実装されている実行経路

```text
source/task
   |
   v
candidate generation (model, optional)
   |
   v
ReasoningArtifact / framework trace
   |
   +--> deterministic validation
   +--> evidence / provenance gates
   +--> trusted verification receipts from deterministic/external oracles
   +--> narrow deterministic framework passes
   +--> contradiction, assumption, causal, and adversarial diagnostics
   |
   v
accept | reject | unknown
```

### 実装済みのプロバイダー非依存グラウンデッド実行ループ

`accept | reject | unknown` は epistemic/policy decision であり、必ずしも product run の恒久的な終端ではない。[ADR-0002](adr/0002-grounded-resolution-and-finalization.ja.md) で定義された provider-neutral core は、bounded resolution と finalization を追加で提供する。

```text
task + harness-owned evidence
          |
          v
candidate generation
          |
          v
materialize + validate + verify + diagnose
          |
          +--> supported enough ----------------------------+
          |                                                 |
          +--> unknown / insufficient support               |
          |          |                                      |
          |          v                                      |
          |     typed resolution request                    |
          |          |                                      |
          |     external evidence / verifier adapter        |
          |          |                                      |
          |          v                                      |
          |     revise / regenerate                         |
          |          |                                      |
          |          +----------> re-run harness -----------+
          |                                                 |
          +--> refuted --> discard/revise --> re-run -------+
                                                            |
                                                            v
                                                       finalization
                                                            |
                                                            v
                                                  claim coverage check
                                                            |
                                                            v
                                              grounded answer | abstain
```

bounded control loop は core に実装されている。既存の diagnostics は typed request を駆動できるが、具体的な web/RAG/database/MCP/human acquisition は core 外の adapter work である。したがって、control protocol が存在するだけで open-world resolution quality を主張することはない。

## 設計原則

1. `unknown` は成功した epistemic outcome である。
2. evidence なしに `known` または `supported` を主張しない。
3. framework は prose-only explanation ではなく typed trace を生成する。
4. deterministic oracle が存在する場合は、model judge より deterministic check を優先する。
5. soft semantic judging は hard validation と分けて明示する。
6. 失敗した pass が、部分的に無効な state のまま黙って継続してはならない。
7. provider/model adapter は交換可能にし、core semantics の外に置く。
8. schema-valid な model output も、validation と acceptance policy が走るまでは candidate にすぎない。
9. live model quality と deterministic contract regression は別々の execution mode である。
10. retrieval または tool output は acquired data であり、デフォルトでは authority ではない。
11. すべての repaired/regenerated candidate は、original と同じ validation と verification boundary を通過する。
12. final renderer は epistemic state を引き上げたり、unsupported factual proposition を導入したりできない。
13. resolution には budget があり、budget exhaustion は fabricated completion ではなく、明示的な unresolved/abstain outcome になる。

## インターフェース

native runtime が correctness boundary である。CLI と eval が最初にサポートする interface である。desktop UI は deferred な薄い inspection client、public embedding API は実利用後にのみ安定化し、MCP は correctness boundary の一部ではなく optional integration adapter とする。

[ADR-0001](adr/0001-interface-and-packaging-boundaries.ja.md) と [ADR-0002](adr/0002-grounded-resolution-and-finalization.ja.md) を参照。

durable policy/session control については [ADR-0003](adr/0003-reasoning-control-plane.ja.md) を参照。

durable control plane は現在2層で実装されている。#27 `ReasoningPolicy` は admissibility/escalation/invalidation を管理し、#28 `ReasoningThread` は typed な append-oriented event と reconstructable checkpoint を記録する。Replay は pure harness-state reconstruction であり、記録済み resolver attempt は再実行しない。interrupted/pending-policy/finalized state は fail-closed であり、immutable finalized history から継続する唯一の方法は fork である。[durable reasoning threads](reasoning-thread.ja.md) を参照。

## 実装言語の境界

first-party の executable と library component はすべて Rust で実装する。native runtime、CLI、evaluation tooling、model adapter、将来の desktop client および optional integration adapter を含む。model provider は外部 service のままで、Rust adapter 経由で到達する。JavaScript/TypeScript runtime は correctness boundary に含めない。

## ランタイム判断の境界

runtime は最初の pass の前と、すべての pass の後に input artifact を validation する。次に policy が valid artifact を `accept | reject | unknown` に写像する。初期 strict policy は explicit contradiction を reject し、`assumed` または `unknown` claim を `unknown` outcome として保持する。この policy は意図的に保守的で、fixture evidence に基づく場合だけ発展させる。

grounded runtime では、この policy result が、run を finalize できるか、typed resolution request を出すべきか、revise/regenerate すべきか、または unresolved で停止すべきかを追加で決める。policy は `unknown` ですぐ停止してもよい。resolution は明示的 capability であり、answer を捏造する義務ではない。

[prior art](prior-art.ja.md) には、runtime dependency を追加せずに検討した外部の design pattern を示す。

## 候補の権威性の境界

model output は finalized `ReasoningArtifact` ではなく `ReasoningCandidate` として表現する。candidate には proposed claim、proposed epistemic state、inference edge が含まれるが、evidence を供給することはできない。runtime は candidate と harness-owned `HarnessInput` を組み合わせ、model が提案した `known`、`supported`、`inferred`、`contradicted` state を当初 `assumed` として materialize する。より強い state を後から確立できるのは harness-owned verification pass だけである。不確実性は安全な epistemic outcome なので、model は `unknown` を保持してよい。

これにより provider が自分の evidence record を捏造したり、claim を supported と自己認証したり、schema-valid な label を出すだけで最終的な contradiction verdict を強制したりすることを防ぐ。

同じ規則を将来の repair/regeneration にも適用する。diagnostic feedback を受けた model はよりよい candidate を提案できるが、replacement candidate は untrusted のまま開始し、repair phase で生成されたという事実から authority を得ない。

## 検証レシートの境界

`VerificationReceipt` は authority-bearing data であり、意図的に `ReasoningCandidate` には含めない。trusted verifier は candidate generation 後にのみ receipt を作成する。推奨する hard-verification contract は typed `Proposition { key, value }` を harness evidence が所有する structured facts に束縛する。evidence qualification requirement のない input は `StructuredFactVerifier` の compatibility behavior を保持する。requirement のある input では、`QualifiedStructuredFactVerifier` が harness-owned temporal/scope/provenance requirement を通して structured fact を filter してから hard receipt を作り、qualified value が複数衝突する場合は receipt を出さない。fact がない、または qualified でない場合は不確実性を保持する。receipt 適用時は authoritative claim text を `key = value` に canonicalize し、model が書いた prose が verifier-endorsed wording として表示されないようにする。exact statement-bound receipt は外部 verifier 向けの保守的な compatibility path としてのみ残す。

receipt は semantic score ではない。caller が指定した verifier に由来する hard verifier result を表す。現在の fixture benchmark は、既知の oracle coverage の下で process correctness を検証するため、明示的な `fixture_oracle` receipt を使う。これは generic reasoning accuracy として報告してはならない。

## 解決境界 — 実装済みコア

resolution layer は unresolved verified state を追加作業の typed request に変換する。request は不足している support を記述するが、不足した fact を発明しない。

想定する request family:

- proposition evidence acquisition;
- causal relation evidence acquisition;
- temporal/scope/provenance qualification;
- deterministic external verification;
- hard refutation 後の candidate revision;
- policy が許す場合の明示的な human review。

runtime は request identity、attempt history、budget、許可された resolver class、verification に戻る state transition を所有する。外部 system は domain-specific acquisition mechanics を所有する。

Web search、retrieval pipeline、database、MCP tool、compiler、test、policy engine、human は resolver adapter として動作できる。その output が authority を得るのは、他と同じ harness-owned evidence または verifier contract に従う場合だけである。retriever が document を返すことは、retrieval の契機となった proposition を verifier が証明することと同じではない。

entity lookup でも同じ境界を使う。candidate list や adapter ranking は plausibility にすぎず、entity identity sufficiency は Harness が決める。no-context では cross-source rank-1 identity gate を維持し、trusted context がある場合は Harness-owned canonical query と deterministic context compatibility を追加で要求する。planner は query suggestion を生成して identity を確定できず、exact Wikibase-QID direct bridge も trusted-context coordinate に限定された observation であって、それ自体は authority ではない。詳細は[エンティティ同一性ゲート](entity-identity-gate.ja.md)を参照。

resolution implementation は、resolver が何かを返したというだけで `unknown` を `supported` に黙って変換してはならない。

`ResolutionResolver` は trusted metadata や receipt を返せない。raw `AcquiredEvidence` は `EvidenceAdmissionPolicy` を通過してから `HarnessInput` に入る。`TrustedResolutionVerifier` は別の authority-bearing interface である。default admission policy は acquired evidence をすべて reject する。run 単位・request 単位の attempt/token/time budget と resolver-class allowlist は runtime が所有する。admitted-evidence または candidate-revision の各 step では、通常の normalization、validation、verification、diagnostic、decision path を再実行する。

具体的な contract と deterministic benchmark は [bounded grounded resolution and finalization](grounded-resolution.ja.md) を参照。

## 最終化境界 — 実装済みコア

finalization は verification や presentation style とは別である。

finalizer は verified artifact state を受け取り、policy に従って grounded answer、qualified partial answer、unresolved result、abstention、または `requires_verification` result を生成する。model を renderer として使ってもよいが、renderer は authority を作れない。

必須の target invariant は **final claim coverage** である。final answer に現れる factual proposition は、supported artifact proposition に対応するか、policy に従って unresolved/assumed として明示されなければならない。renderer が新しい factual proposition を導入した場合、その proposition は grounded output に現れる前に通常の candidate/verification loop に戻さなければならない。

artifact-global `Reject` に対する target-local の狭い recovery も存在する。これは global verdict を promote も rewrite もしない。exact requested target は、matching target claim のすべてに direct evidence-bound trusted `Supported` verification があり、かつ typed artifact が rejected non-target state から構造的に分離されていることを示す場合に限り `QualifiedPartialAnswer` として公開できる。条件は target-local contradiction/qualification/hard adversarial finding がないこと、same-key blocker がないこと、untyped problematic claim がないこと、unresolved または contradicted blocker への shared evidence や inference/dependency path がないこと。contradicted blocker 自体には direct evidence-bound trusted contradiction receipt が必要である。typed dependency に不確実性があれば fail closed する。

これにより `ReasoningArtifact` を source of truth とし、fluent final-generation step が run 前半の correctness work を取り消すことを防ぐ。

## 限定的な決定論的フレームワークチェック

Five Whys restatement pass は、提案された cause が effect を実質的に言い換えていると意図的に狭い lexical heuristic が認識した場合にのみ causal edge を削除する。conclusion は不確かなまま残る。これにより string heuristic が semantic causal authority になるのを防ぐ。

## 候補正規化の境界

`ReasoningCandidate` は untrusted syntax であり、trusted reasoning state ではない。構造的に無効な inference suggestion（例えば premise の欠落や、存在しない claim への参照）は artifact validation 前に削除し、`candidate_diagnostics` に記録する。これは silent repair ではない。artifact には削除したすべての edge の inspectable record を残す。claim 自体は通常の downgrade と hard-verification boundary を通るため、normalization が claim を promote したり authority を作ったりすることはできない。

## 敵対的探索の境界

`AdversarialDetector` は `contradiction | counterexample` kind と `hard | soft` strength を持つ typed `AdversarialFinding` record を生成する。discovery は observational である。finding は artifact に記録されるが claim epistemic state を変更せず、直接 `reject` を強制できない。hard authority は deterministic `Verifier` implementation と trusted verification receipt に残る。初期の `StructuredFactConflictDetector` は harness-owned structured fact だけを読む。model-backed semantic discovery は calibrated soft-judge contract として別に利用でき、model が finding を報告しても advisory のままであり、authority には independent hard verifier がなお必要である。

この分離により、model が生成した contradiction label や counterexample suggestion が自己認証 evidence になることを防ぐ。

## 推論ポリシーと無効化の境界

`ReasoningPolicy` は admissibility と escalation を制約するが、truth は決めない。global/domain/run policy layer は authority、scope、derived-support capability、resolver-class permission について保守的に合成される。soft semantic finding は追加作業を要求できるが、policy rule が evidence、receipt、hard finding、epistemic promotion、verdict authority を作ることはできない。

policy change は history を mutate せず、新しい `ReasoningArtifact` snapshot を作る。hard state は、その authority が effective policy 下で再構築可能な場合にのみ保持される。invalidated receipt は claim、dependent inference edge、downstream claim、finalization へ伝播する。影響を受ける edge は新しい accepted snapshot から削除され、policy-sensitive qualification/assumption diagnostic が再計算され、`StrictAcceptancePolicy` が再評価される。旧 artifact は将来の thread/history ownership のため変更されない。

`constrain_resolution_policy` は #22 を再利用し、resolver class と required evidence authority を厳しくすることだけができる。[reasoning policy and dependency invalidation](reasoning-policy.ja.md) と [ADR-0003](adr/0003-reasoning-control-plane.ja.md) を参照。

## ソフトセマンティック判定器の境界

`SoftDiagnosticJudge` は明示的に non-authoritative な discovery/calibration boundary である。typed diagnostic request と stable judge/model/configuration identity に結び付いた `finding | no_finding | abstain` observation を出す。`SoftSemanticFinding` には verification receipt、verdict、epistemic-state mutation、hard-strength field が意図的に存在せず、初期 calibration implementation では `ReasoningArtifact` に保存しない。

calibration では precision/recall、decision coverage、disagreement、abstention、pairwise categorical agreement、nominal Krippendorff alpha を final harness correctness と分けて報告する。ambiguous label は保持するが positive/negative precision/recall から除外する。abstention は明示的に残し、finding に majority-vote せず alpha では missing data として扱う。

implemented policy/thread layer は soft observation を記録したり追加 evidence を要求するために使ったりできるが、hard authority を作れるのは既存の harness-owned evidence/verifier boundary だけである。model-backed soft judging にも同じ規則を適用し、自分の output を promote できない。[soft semantic-judge calibration](semantic-judge-calibration.ja.md) を参照。

default semantic runtime profile は `semantic-decidability-d3-v1` である。model output はまず harness-owned R2 decision-only materialization boundary を通り、次に deterministic explicit-typed-preconditions gate がその soft decision を保持するか `abstain` にする。以前に characterise された `soft-semantic-v3` profile は明示的な rollback path として残る。runtime profile の選択が変えるのは advisory behavior だけで、どちらの profile も trusted evidence、verification receipt、hard finding、epistemic promotion、verdict authority を作れない。[semantic runtime stabilization and adoption](semantic-runtime-stabilization.ja.md) を参照。

## 仮定診断の境界

`HarnessInput.assumptions` は harness-owned input であり、意図的に `ReasoningCandidate` から除いている。これは task が premise として使ってよい proposition を示すが、独立検証済みだとは主張しない。candidate が評価するよう task から求められる proposition を示す `hypotheses` とは異なる。

`AssumptionInspector` は trusted verification pass 後に inference premise として実際に使われた proposition を調べる。`known`/`supported` premise は trusted であり、`inferred` premise は inference chain の末端が trusted support または明示的 input assumption の場合にだけ derived support と数える。したがって candidate 自身の `inferred` label だけでは不十分である。trusted support も explicit input assumption もない typed premise は hard `unsupported_premise` process finding を生む。proposition binding のない premise は soft `unbound_premise` finding を生む。finding は observational のままで、evidence、verification receipt、verdict authority を作れない。

resolution loop では、これらの finding が resolution request や candidate revision の動機になることはあるが、actionable になったことで authority が増すことはない。

## エビデンス適格性評価の境界

`Evidence.metadata`、`EvidenceRequirement`、`EvidenceAuthorityPolicy` は harness-owned であり、`ReasoningCandidate` にはない。domain-specific な source name を core logic に埋め込まず、structured fact が明示された time、scope、minimum opaque authority rank に適用可能か runtime が試験できる。deterministic mismatch は hard finding であり、metadata 欠如は soft/unknown のまま残る。

Evidence qualification 自体は observational だが、built-in structured verifier は hard receipt を作る前に同じ requirement を消費する。これにより stale、out-of-scope、authority 不足の fact が黙って `supported`/`contradicted` になるのを防ぐ。qualified value が衝突すると diagnostic conflict を生成し、built-in hard receipt は作らない。明示的な external trusted receipt は独立した oracle compatibility boundary として残り、この layer が自動的に再解釈することはない。

implemented resolution loop では、新たに取得した evidence も unknown を解決する前に同じ qualification boundary を通る。したがって retrieval は、関連して見える record を返しただけで time、scope、provenance policy を迂回できない。

scope semantics、authority-policy rule、diagnostic 間の相互作用は [temporal, scope, and provenance evidence qualification](evidence-qualification.ja.md) を参照。

## エビデンスを考慮した因果診断の境界

`CausalInspector` は Five Whys inspection を lexical restatement の先まで拡張するが、verdict authority にはならない。typed causal relation を cause proposition(s) -> effect proposition として canonicalize し、その relation を明示的 provenance を持つ harness-owned `CausalEvidence` にだけ照合する。exact support は edge を `supported` にでき、exact trusted refutation は `refuted` にできる。association-only evidence、partial support、reverse-direction support、conflicting evidence、relation evidence 欠如、不完全な proposition binding は、soft diagnostic とともに `unknown` のままである。

Causal inspection は observational であり、claim epistemic state を変更せず、verification receipt を作らず、final `accept | reject | unknown` policy result を直接変えない。既存の lexical Five Whys cleanup は狭い deterministic fast path のままだが、cleanup は問題のある exact inference edge に局所化され、独立に hard-supported な claim を downgrade できない。`fixtures/causal/` の dedicated deterministic corpus は original claim-verdict benchmark と repeated-trial correctness denominator から分けて評価する。

詳細な contract と deferred scope は [evidence-aware causal diagnostics](causal-reasoning.ja.md) を参照。

## メタモルフィック評価の境界

metamorphic transform は evaluation layer にあり、runtime authority boundary にはない。transform は set-like record の順序を変えたり、referential ID を一貫して remap したり、明示的に unrelated な control fact を追加したりできる。ただし proposition meaning、trusted verification conclusion、causal direction/membership、その他 oracle semantics を変更してはならない。

evaluator は semantic diagnostic signature と raw finding ID を別々に比較する。これは valid な stable-ID remap により generated diagnostic identifier が変わっても同じ hard finding を保持しうるため必要である。final-verdict invariance、hard-finding invariance、soft-finding stability、typed diagnostic-status invariance は独立して報告し、original benchmark correctness denominator の代用にはしない。

現在の transform contract は [metamorphic reasoning robustness](metamorphic-testing.ja.md) を参照。

## バージョン管理済みコーパス測定の境界

`CorpusManifest` は evaluation metadata であり、runtime authority ではない。stable suite-prefixed case identity、category/difficulty strata、scoring mode、provenance/redistribution notes、contamination posture、lifecycle status を model/provider output から独立して固定する。`score_compatibility_id` は direct score-comparison compatibility を明示する。

recorded claim stratification は既存の `BenchmarkComparison` aggregation を再利用するため、correctness semantics を再定義できない。live run は corpus identity を記録するが、repeated または incomplete trial 間で category/difficulty score を pool しない。将来の resolution variant は stable base case ID を再利用し、recovery metric を original denominator の追加として扱う。置き換えにはしない。

compatibility と change rule は [versioned benchmark corpus](corpus-versioning.ja.md) を参照。

## 反復診断測定の境界

repeated diagnostic aggregation は evaluation/reporting boundary であり、verifier ではない。`DiagnosticSignal` は adversarial finding、candidate-normalization code、causal finding/reason observation、assumption signal、evidence-qualification finding を記録するが、それらに新しい authority を与えない。`stability.diagnostics` は final correctness stability の内部ではなく、横並びで serialize する。

operationally complete な trial だけが diagnostic frequency と count distribution に寄与する。incomplete provider trial の partial な successful observation は diagnostic absence と解釈せず、excluded observation として報告する。confidence interval は minimum complete-observation threshold に達した後、文書化された 95% Wilson score method だけを使う。exact count と denominator は常に保持する。

resolution-loop evaluation は diagnostic stability と通常の verdict accuracy の両方から分離する。recovery rate は unsafe-final-answer rate、final claim coverage、resolution cost、明示的な exhaustion count と併せて初めて有用になる。
