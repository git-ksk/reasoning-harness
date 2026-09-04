# Temporal、scope、provenance に基づく evidence qualification

evidence-qualification layer は、harness-owned evidence が、harness が要求する時点・scope・source-authority level で proposition に適用可能かを判定する。evidence の retrieval、実世界 source の名称による ranking、欠落 metadata の推測は行わない。

## Harness が管理する契約

`Evidence.metadata` には次を含められる。

- `temporal`: inclusive Unix-second bounds による任意の validity window
- `scope`: coverage が `any` または明示的 value set である provider-neutral applicability dimension
- `provenance_class`: 意味を harness が与える opaque class label

`HarnessInput.evidence_requirements` は proposition key ごとに1つの qualification requirement を bind する。`as_of_unix_seconds`、required scope、minimum authority class を指定できる。proposition key ごとに最大1 requirement なので、同じ key の structured fact は一意な qualification context で評価される。

`HarnessInput.authority_policy` は opaque provenance class を integer rank に対応付ける。core は特定 source の強弱を知識として内蔵しない。domain-specific source taxonomy は core 外に置く。

これらはすべて harness-owned で `ReasoningCandidate` には存在しない。model は supplied context を観測できるが、evidence metadata、requirements、provenance class を作成・変更・昇格できない。

## Qualification の意味論

required as-of time が evidence validity window 内なら有効である。`effective_from` 前は `not_yet_valid`、`effective_until` 後は `stale`。temporal metadata がなければ推測で stale/current にせず `unknown`。

required scope は各 dimension で evidence coverage の subset でなければならない。disjoint は `scope_mismatch`、部分 coverage または narrower evidence だけで支えた universal requirement は `scope_expansion`。required scope metadata の欠落は `unknown`。

provenance class は harness policy の rank が required class 以上なら qualify する。低い rank は `insufficient_authority`、欠落または未登録 provenance は `unknown` とし、core は順序を推測しない。

deterministic mismatch は hard finding、binding 不足は構造情報が足りず mismatch を証明できないため soft finding である。

## 検証との連携

evidence qualification は observational だが、evidence requirements がある入力では built-in structured-fact hard-verification path も qualification-aware になる。stale、not-yet-valid、scope-mismatch、scope-expansion、insufficient-authority、metadata-insufficient の record は hard receipt から除外する。残る evidence がなければ receipt を出さず claim は uncertain のままにする。

同じ key の qualified evidence が conflicting value を持つ場合、hard conflict finding を出し qualified verifier は receipt を withheld する。qualified evidence の衝突を hard support/contradiction verdict にはしない。

evidence requirements がない入力は backward compatibility のため従来の `StructuredFactVerifier` を使う。明示的な `TrustedVerificationPass` receipt は別の external-oracle compatibility boundary であり、qualification によって自動 downgrade しない。

## 他の diagnostics との関係

Assumption diagnostics は hard verification 後に動くため、適用可能な evidence requirement を満たす場合だけ built-in verifier が premise を trusted support にできる。unqualified evidence は derived-support chain を bootstrap できない。

`CausalEvidence` は別の harness-owned causal relation contract である。generic `Evidence.metadata` qualification は causal support/refutation semantics を書き換えない。#16 は2つの authority type を merge しない。

evidence-qualification finding は独自の `DiagnosticSignal` family として repeated-trial reporting にも出るが、final verdict accuracy と causal-edge-quality denominator の外側である。

## 決定論的コーパス

`fixtures/evidence-qualification/` は credential-free の8-case regression corpus で、exact qualification、stale、not-yet-valid、disjoint scope、unsupported scope expansion、insufficient authority、conflicting qualified evidence、missing metadata を扱う。qualification behavior だけを測り、historical 20-case claim-verdict / 8-case causal denominator は変更しない。

## 対象外

open-world retrieval、web search、generic RAG orchestration、automatic recency-based truth selection、domain-specific source ranking、model-generated provenance certification は行わない。
