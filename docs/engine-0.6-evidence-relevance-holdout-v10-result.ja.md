# Engine 0.6 evidence relevance independent holdout v10 result

Status: immutable canonical FAIL。

Freeze tag: engine-0.6-evidence-relevance-holdout-v10-freeze
Freeze commit: 4aa3aac37ea1a4110e4bf69b498661bc3599a36a
Canonical run: 37085574010、attempt 1 only
Cases: 26（Relevant 8 / Irrelevant 10 / Ambiguous 8）
Required providers: Mistral + Google + Groq
Effective qualification: v11
Materialization: v23
Final gate: FAIL

この結果は immutable。rerun / rescore / relabel / freeze tag の移動・再作成 / PASS への再解釈は禁止。

## Provider results

Mistral / ministral-8b-latest:
- operational 26/26、provider failure 0
- authority failure 0
- identity/risk failure 0
- materialized exact 26/26
- wrong-target Relevant 0
- utility miss 0

Google / gemini-3.5-flash-lite:
- operational 26/26、provider failure 0
- authority failure 0
- identity/risk failure 0
- materialized exact 26/26
- wrong-target Relevant 0
- utility miss 0
- v10h15_negative_availability_feature_support_non_frame だけ expected different_relation に対し effective unresolved
- forbid_requested 内の安全な保守差で、final materialization には影響なし

Groq / openai/gpt-oss-120b:
- operational 26/26、provider failure 0
- authority failure 0
- identity/risk failure 0
- materialized exact 25/26
- wrong-target Relevant 0
- false relevance rejection 0
- relevant left Ambiguous 0
- utility miss 1
- relation-label difference は v10h15 / v10h18
- terminal miss は v10h18_negative_exact_target_numeric_non_frame、expected Irrelevant -> materialized Ambiguous

## v10h18 adjudication

v10h18 の expectation は holdout drift ではない。

Task: What hard limit applies to Ivory Stream?
Candidate: Ivory Stream processed 93 jobs per project yesterday during routine telemetry.

freeze 前の successor-v9 development contract で、同じ exact-target numeric-observation family の sv9d_19_exact_target_model_only_limit は既に different_relation / Irrelevant と固定済み。sv9d_07_limit_numeric_observation も different_relation を期待する。

Groq は:
- proposal: exact target / unresolved relation
- local qualification: exact_target / relation_absent / none
- effective qualification: exact_target / relation_absent / none
- materialized disposition: Ambiguous

これは unsafe authority promotion ではなく safe over-abstention。ただし、単なる telemetry 観測値は hard limit の根拠ではなく別relationの観測なので、frozen v11/v23 contract に対する実際の utility / coverage miss である。

## Final gate interpretation

Canonical result:
- operational completeness: PASS
- correctness hard gate: PASS
- qualification safety gate: PASS
- materialization gate: FAIL
- utility gate: FAIL
- overall: FAIL

unsafe requested-relation promotion、wrong-target Relevant、identity violation、provider/protocol failure は全providerで0。

残存gapは exact-target non-frame negative relation evidence の cross-provider robustness。v11 は positive authority を安全に拒否できているが、provider が明確な different_relation ではなく relation_absent を返した場合、v23 は Ambiguous に残り得る。

## Next research direction

v10 を tuning data にせず、label変更やrerunもしない。

successor は v10 と独立した fresh development surface で、exact-target non-frame observation 向けの monotone / one-sided negative-relation mechanism を検討する。
- requested-relation authority を生成しない
- genuine missing/truncated relation evidence の Ambiguous を維持
- 別relationの明示的観測と、requested relation の単なる欠如を区別
- 新holdout前に frozen historical relation controls を全replay
- successor semantics freeze 後にのみ fresh independent holdout を作る

Issue #468 は open のまま維持する。
