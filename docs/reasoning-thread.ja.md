# Durable reasoning thread と決定論的リプレイ

Issue #28 は ADR-0003 の durable control-plane contract を実装する。ただし Reasoning Harness を conversation store や汎用 agent session manager にはしない。

## 境界

`ReasoningThread` が永続化するのは、明示的に型付けされた runtime state だけである。

- task identity と task text;
- stable candidate IDs/replacement lineage を持つ untrusted `ReasoningCandidate` snapshots;
- accepted `ReasoningArtifact` state と verdict;
- `ReasoningPolicy` versions と deterministic #27 invalidation transitions;
- 非権威的な observation としての soft-judge observations;
- すでに実行された #22 `ResolutionAttempt` records;
- checkpoint、interrupt/resume/fork の control events;
- finalization results.

hidden chain-of-thought は contract に必要なく、表現もしない。

## 追記指向のイベント

Events は monotonic sequence numbers、stable event IDs、任意の causation IDs を持つ。現在の typed families は次のとおり。

- `task_received`;
- `candidate_recorded`;
- `artifact_accepted`;
- `soft_finding_recorded`;
- `resolution_attempt_recorded`;
- `policy_changed`;
- `state_invalidated`;
- `checkpoint_created`;
- `interrupted`;
- `resumed`;
- `forked_from`;
- `answer_finalized`.

policy change とその invalidation は意図的に別 event である。その間の replay は `needs_reevaluation` を報告し、checkpoint/finalization を拒否する。invalidation event は #27 `apply_reasoning_policy` の deterministic re-run と完全一致しなければならず、event log が別の authoritative artifact を捏造することはできない。

## チェックポイント

`ReasoningCheckpoint` には次が含まれる。

- stable checkpoint and thread IDs;
- thread schema version;
- event sequence;
- active policy version, when present;
- 再構成可能な harness-owned snapshot.

checkpoint は active accepted-state boundary でのみ作成できる。保存 snapshot は `checkpoint_created` event 時点の deterministic replay と完全一致しなければならない。変更された checkpoint や stale checkpoint は replay に失敗する。

snapshot は過去の resolution-attempt records と soft observations を保持する。これにより control/accounting context を保ちながら adapter を再実行しない。

## 中断と再開

Interrupt には最新の safe checkpoint が必要である。interrupted 後は thread が凍結され、`resume` だけを受け付ける。interrupted snapshot は新たに verified または finalized されたものとは扱わず、finalization text は消去する。

Resume は checkpoint snapshot を正確に復元し、thread を `active` に戻す。external resolver/tool の side effects は replay しない。`resolution_attempt_recorded` は過去の typed data にすぎない。

## フォーク

Fork は non-destructive である。次を持つ新しい thread ID を作成する。

- same root lineage ID;
- parent thread ID;
- source checkpoint ID;
- copied accepted checkpoint snapshot.

source thread/history は変更されない。stable candidate identity は checkpoint から引き継がれるため、fork 内の修復・置換 candidate は置き換える candidate を参照できる。

finalized source は immutable だが、呼び出し側は以前の safe checkpoint から fork して新しい lineage を続けられる。

## ポリシーとの連携

thread に active `ReasoningPolicy` がある場合、新たに記録する accepted artifact はその policy の下ですでに admissible でなければならない。replay は #27 を deterministic に再適用し、policy constraints を迂回しようとする event を拒否する。

policy transition では、replay が直前の accepted artifact と以前の policy から #27 `ReasoningPolicyTransition` 全体を再計算する。記録された transition は、それが current state になる artifact/verdict より前に完全一致しなければならない。

## 永続化バックエンド

Core は意図的に、serializable contract と抽象的な `ReasoningThreadStore` load/save boundary だけを提供する。filesystem、database、cloud service、retention policy は含まない。

large-payload deduplication/content addressing は、authority semantics of replay を変えずに将来の backend または adapter で実装できる。

## リプレイ安全性の不変条件

1. event sequence と event IDs は deterministic に検証される;
2. checkpoint schema/thread/policy identity が検証される;
3. interrupted threads は resume 以外で進められない;
4. pending policy change は matching deterministic invalidation 以外で進められない;
5. finalized threads は immutable であり、作業継続には fork が必要;
6. resolver attempt records は replay 中に resolver を呼び出さない;
7. soft findings は observations のままで、verification authority にはなれない;
8. policy transitions は serialized data を信頼せず再計算される;
9. hidden model reasoning は persistent state の一部ではない。

regression suite は credential-free であり、checkpoint/resume equivalence、fork lineage、policy/invalidation replay、tamper rejection、interrupted/finalized gates、resolver-side-effect non-replay、hidden-chain-of-thought fields の不在を検証する。
