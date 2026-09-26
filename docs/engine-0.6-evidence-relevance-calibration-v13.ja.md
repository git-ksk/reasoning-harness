# Engine 0.6 candidate: evidence-target relevance calibration v13

状態: immutable v12 canonical evidence の successor、pre-freeze。v1-v12 の fixture / tag / observation / score は不変。first/only frozen v13 canonical calibration が PASS するまで independent holdout は作成しない。

## v13 の目的

v12 は v11 の open-world overblocking を大幅に減らしたが PASS には届かなかった。required Mistral は 73/73 完走、provider failure 0、qualification-risk miss 0、wrong-target relevance retention 0、false relevance rejection 0 だった一方、materialized exact は 55/73、utility miss は18。18 miss のうち17件は expected Irrelevant -> Ambiguous、1件は expected Relevant -> Ambiguous。さらに18件中16件で frozen expected relation binding と不一致だった。

post-observation の静的監査では frozen v12 corpus 自体にも別の specification 問題を確認した。一部の explicit-local-absence case が v12 prompt の定義と矛盾していた。v12 label は historical evidence として変更せず、v13 は observation 前に明示した annotation protocol で新しい calibration surface を再注釈する。

v12 Groq では別系統の operational failure も露出した。JsonSchema request が HTTP 400 でも provider capacity を消費し得る一方、成功 usage telemetry がないため local token pacer がその消費を反映できない。直後の strict-JSON Text fallback が HTTP 429 を受け、retry-after が shared 60秒 case deadline を超えた。v13 では Groq の primary / qualification 両 stage を strict raw-JSON Text primary に固定する。Mistral / Google は JsonSchema primary + bounded strict Text fallback を維持する。

## 研究アンカー

設計の主根拠は repository で観測した failure。以下の論文は supporting evidence として使い、設計を自動的に正当化する authority としては扱わない。

- Weir et al., [Enhancing Systematic Decompositional Natural Language Inference Using Informal Logic](https://aclanthology.org/2024.emnlp-main.531/) (EMNLP 2024): entailment annotation protocol の明確化と一貫性。v13 は model observation 前に field-level annotation rule を固定する。
- Chen et al., [PropSegmEnt: A Large-Scale Corpus for Proposition-Level Segmentation and Entailment Recognition](https://aclanthology.org/2023.findings-acl.565/) (Findings ACL 2023): 一つの文書単位に複数 proposition が含まれ得る。v13 は target identity と relation kind を別 atomic proposition として扱う。
- Srinivasan et al., [Selective “Selective Prediction”: Reducing Unnecessary Abstention in Vision-Language Reasoning](https://aclanthology.org/2024.findings-acl.767/) (Findings ACL 2024): selective system の unnecessary abstention。v13 は generic uncertainty ではなく observable local blocking cue を要求する。
- Xu et al., [Do Language Models Mirror Human Confidence? Exploring Psychological Insights to Address Overconfidence in LLMs](https://aclanthology.org/2025.findings-acl.1316/) (Findings ACL 2025): assessment と answer production の分離。v13 も independent qualification と Harness-owned materialization を分離する。
- Geng et al., [Generating Structured Outputs from Language Models: Benchmark and Studies](https://arxiv.org/abs/2501.10868) (2025): structured output の挙動が schema / constrained-decoding implementation に依存する。v13 では Groq transport を provider-specific precommit とする。

これらは v13 PASS を保証しない。pre-observation hypothesis を明確化し、自由度を減らすために使う。

## Atomic binding proposal v3

primary model は advisory な target_binding と relation_binding の2 fieldだけを返し、それぞれ exact | different | unresolved。

両 field は明示的に直交させる。target_binding は substantive material がどの target / entity に local scope されるか。relation_binding は proposition がどの relation kind を表すかであり、その relation の owner が target か sibling かとは独立。

したがって:
- sibling target + requested relation => different / exact
- sibling target + another relation => different / different
- exact target + another relation => exact / different
- generic / explicit-local-absence で substantive relation proposition がない => relation unresolved

target difference だけを理由に relation difference を推論してはならない。

## Local qualification v3

independent guard は6つの atomic field を返す:
- target_support: supported | not_supported | unresolved
- relation_support: supported | not_supported | unresolved
- identity_mapping_cue: absent | present
- ownership_scope_cue: absent | present
- context_gap_cue: absent | present
- explicit_local_absence: present | absent | unresolved

v7 materializer では旧 risk の Present / Unresolved はどちらも fail-closed blocker だった。Harness consequence が同じなのに model へ3値区別を要求することは label complexity を増やすため、v13 は concrete local blocker が観測できるかの2値 cueへ変更する。

cue present は candidate 自体に具体的 signal がある場合だけ: uncertain rename / successor / conflicting identity mapping、shared / unassigned row・section・value ownership、または binding に必要な local context が visible に clipped / truncated / omitted。

generic open-world uncertainty は cue ではない。明確な generic passage や「target-specific information がない」という local statement は local-absence evidence であり、それ自体を context gap にはしない。

## Materialization v8

Harness authority は fail-closed のまま。

Hard Relevant は primary exact/exact、guard supported/supported、blocking cue 全て absent、policy が要求する Harness-owned identity anchor が必要。

Hard Irrelevant は次のいずれか:
- primary target different + guard target not_supported + cue absent
- primary target non-exact + target not_supported + explicit local absence present + cue absent
- primary exact target + relation different + guard target supported + relation not_supported + cue absent

blocking cue、CanonicalUrl-only floor、missing proposal / guard、strict identity failure、two-key disagreement は Ambiguous のまま。

変更点は model-facing semantic decomposition であり、Harness authority boundary ではない。

## Transport

Mistral / Google は JsonSchema primary、structured capability unsupported または malformed response の場合だけ strict raw-JSON Text fallback 1回。response 全体が typed contract に直接 parse できること。

Groq は各 semantic stage で strict raw-JSON Text を primary / only transport とする。preceding JsonSchema request はなく、malformed Text は typed protocol failure。semantic retry はしない。

全 provider で JSON extraction、Markdown stripping、field synthesis、fuzzy repair、semantic retry、third model call は禁止。primary + qualification は同じ 60,000ms case deadline を共有し、consecutive operational failure 2件で circuit open。

## Calibration corpus

v13 は synthetic 81 cases。先頭73件は v12 calibration material を v13 protocol で successor re-annotation したもの。historical v12 expected label は変更しない。v12 で発見した explicit-local-absence annotation contradiction 2件は、この新しい v13 corpus のみで修正する。

v13 live observation 前に fresh 8 cases を追加: exact target + requested relation、sibling target + requested relation、sibling target + different relation、explicit generic local absence、uncertain rename + same relation、shared row + same relation、clipped local relation context、exact target + different relation。

Expected は Relevant 24 / Irrelevant 29 / Ambiguous 28。blocking cue expected case は28件。explicit local absence は9件。production motivating product は tuning fixture に含めない。

## Canonical roles / acceptance

Required:
- Mistral ministral-8b-latest
- Groq openai/gpt-oss-120b

Full non-gating replication:
- Google gemini-3.5-flash-lite

required arm は個別に 81/81 operational completion、provider failures 0、provider-attempt telemetry complete、local qualification expected/invoked 81/81、blocking-cue miss 0、wrong-target relevance retention 0、false relevance rejection 0、expected Relevant left Ambiguous 0、utility miss 0、materialized exact 81/81 を満たす必要がある。

proposal exactness、qualification full-field exactness、spurious cue block は diagnostic。spurious cue による user-visible overblocking は final disposition exactness / utility hard gate で落ちる。

first/only frozen v13 canonical は rerun / rescore しない。canonical PASS の場合のみ fresh independent holdout を作成する。holdout PASS 後のみ runtime integration acceptance へ進む。PR #466 は全 stage PASS まで Draft のまま。
