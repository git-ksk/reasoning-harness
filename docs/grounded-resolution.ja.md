# 制限付き根拠ベース解決と確定処理

Issue #22 は、既存の reasoning harness の周囲に provider-neutral control loop を追加する。この loop は unresolved typed state を bounded resolution request に変換し、明示的な evidence-admission boundary 経由でのみ取得データを受け入れ、通常の verification pipeline を再実行し、typed factual-claim coverage を確認してから出力を確定できる。

これは runtime protocol であり、web-search や RAG の実装ではない。

## 権限の境界

4つの boundary は意図的に分離する。

1. `ResolutionResolver` は acquisition、candidate revision、human-review routing を行う。raw `AcquiredEvidence` は返せるが、raw acquired evidence に `EvidenceMetadata` はなく、trusted evidence ではない。
2. `EvidenceAdmissionPolicy` は harness 所有である。取得 evidence に trusted metadata を付与できるが、その際 acquired ID、source、observation、structured facts を変更する admission implementation は runtime が拒否する。
3. `TrustedResolutionVerifier` は別の authority-bearing interface である。generic resolver は `VerificationReceipt` を作れない。trusted receipt も evidence binding を含む既存の receipt validation contract に従う。
4. `FinalAnswerRenderer` は untrusted final-answer candidate を生成する。`finalize_answer` は text を grounded output として出す前に、すべての typed factual claim を verified artifact state と照合する。

Default evidence admission policy は `RejectAllEvidenceAdmission` である。したがって retrieval が偶然 authority になることはない。

## Resolution リクエスト

`ResolutionRequest` は stable request ID、typed reason、target、requested resolver class、任意の per-request attempt/token/time budget を持つ。

Target は proposition、causal relation、evidence-qualification requirement、claim revision、明示的な human review を表せる。Default planner はまず `ReasoningArtifact.hypotheses` と `evidence_requirements` の harness-owned unresolved target を検討し、exact evidence requirement があれば `EvidenceQualification` target として保持する。その後で unsupported-premise finding、その他の evidence-qualification finding、unresolved generated claim を検討する。この順序により、正確に要求された target を試す前に無関係な candidate claim が bounded resolution budget を消費することを防ぐ。model prose や fuzzy proposition matching から target を推測することはない。既に exact な `Known`/`Supported` target は再要求せず、contradiction は既存の reject/revision policy に従う。Causal target は provider-neutral contract の一部だが、`CausalEvidence` は別の observational contract なので automatic causal-evidence acquisition は延期されている。

Target priority は authority を変えない。Resolver output は configured admission boundary と通常の verification pipeline を通過して初めて epistemic state を変更でき、temporal/scope/authority qualification は exact な harness-owned requirement に付属する。

## 制限付き実行

`GroundedResolutionPolicy` は run 全体の maximum attempts、added-token budget、adapter が報告する elapsed-time budget、allowed resolver classes、required evidence authority class、hard refutation が candidate revision を要求できるか、human review の可否、qualified-partial finalization policy を管理する。

Per-request budget は別に強制され、`GroundedResolutionOutcome.request_usage` は request 単位の attempt/token/time accounting を保持する。Budget exhaustion は epistemic state を変えず、現在の verified state を保ったまま `exhausted` で終了する。

Terminal status は `resolved_supported`、`resolved_qualified`、`resolved_refuted`、`exhausted`、`unavailable`、`human_review_required` である。

## 再検証不変条件

状態を変えるすべての contribution は同じ correctness path に戻る。

- admitted evidence は harness input に追加し、candidate を materialize して再検証する
- candidate revision は untrusted candidate だけを置き換え、最初から normalize、validate、verify、diagnose、decide する
- trusted verifier receipt は通常の trusted-receipt pass を通る
- evidence-qualification requirement は保持または強化し、新規 evidence が hard structured-fact receipt を作れるのはその後だけである

従って、関連しそうな data を返すだけでは unknown は解決されない。

## 確定処理の適用範囲

Finalization は raw provider prose ではなく `ReasoningArtifact` に対して動作する。`grounded` の final claim は known または supported の typed artifact proposition と一致しなければならない。`uncertain` claim も artifact proposition に対応し、admissible かつ contradicted でない epistemic state でなければならない。

Renderer が新しい factual proposition を導入した場合、finalization は `requires_verification` を返し、その text を保留する。Bounded runtime はその proposition を新しい harness-owned hypothesis にして resolution と通常の verification に送り、grounded output として現れるのを許す。

Successor target-local finalizer は artifact-global verdict が `Reject` でも exact requested target を保持できるが、明示的に qualified target-only result とする。Eligibility は通常の target recovery より厳格で、すべての target claim は evidence-bound trusted receipt に直接 `Supported` され、rejected/unresolved non-target claim は typed でなければならない。同一 key の blocker、target qualification/hard adversarial signal、target contradiction、shared evidence、malformed dependency edge、target と問題のある non-target state の typed inference/dependency path が1つでもあれば recovery を無効にする。Contradicted blocker 自身も evidence-bound trusted contradiction receipt に裏付けられなければならない。Global `Reject` と artifact history 全体は変更しない。

現在の default は deterministic `CanonicalFinalAnswerRenderer` である。Model-backed renderer は将来同じ interface を実装できるが、authority は得ない。

## 制御された解決ベンチマーク

`fixtures/resolution/` には stable corpus-v1 base identity に紐づく deterministic resolution variant がある。初期9 scenario はすべて `claim:missing-evidence` を再利用し、新規 supporting evidence、explicit refutation、stale evidence、wrong-scope evidence、insufficient-authority evidence、conflicting evidence、resolver result なし、malformed resolver output、valid-looking but untrusted resolver output を対象とする。

```bash
cargo run -p reasoning-harness-cli -- eval-resolution fixtures/resolution --format human
```

Resolution aggregate は通常の correctness と repeated diagnostic stability とは別で、initially-unknown recovery、unsafe emitted final answer、blocked unverified finalization、final factual-claim coverage、terminal distribution、attempts、adapter-reported token/time cost を報告する。

Committed deterministic baseline は9 scenarioすべて合格で、unknown-to-supported recovery 1、explicit refutation 1、unknown を保持する exhausted 7、unsafe final answer 0、typed final-claim coverage 完全となる。これは process regression test であり、open-world resolver quality の empirical evidence ではない。

## 保留中の統合範囲

Core は generic web crawler、RAG pipeline、database resolver、MCP resolver、human-review backend、provider-specific resolution policy を出荷しない。実 integration は adapter contract を実装し、同じ admission/verifier boundary を維持しなければならない。Live multi-provider resolution research も deterministic CI baseline とは別である。
