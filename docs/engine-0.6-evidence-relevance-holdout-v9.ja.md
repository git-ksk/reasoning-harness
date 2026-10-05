# Engine 0.6 evidence relevance holdout v9

Status: corpus-freeze candidate。successor-v9 semantics と修正版 V9 runner は frozen。fresh 26-case holdout-v9 corpus を author し offline validate 済み。provider observation はまだ0。

## Frozen semantic input

- successor semantics tag: `engine-0.6-evidence-relevance-successor-v9-semantics-freeze`
- tag target: `3e49a9fd2f7827b7c707516d2150e2f0f16e9e17`
- effective qualification: v11
- materialization: v23
- historical v9/v22 predecessor functionsは frozen / unchanged

## Runner wiring

dedicated binary は `crates/reasoning-harness-cli/src/bin/evidence_relevance_holdout_v9.rs` の `reason-evidence-relevance-holdout-v9-study`。

corpus authoring 前に V9 profile を以下で固定する。

- configuration: `evidence-relevance-live-holdout-v9`
- suite: `evidence-relevance-holdout-v9`
- annotation protocol: `evidence-relevance-effective-qualification-v11`
- fixed core: `evidence-relevance-fixed-core-v9`
- expected relative directory: `fixtures/evidence-relevance-holdout-v9`
- expected case count: 26
- checkpoint profile: holdout / complete 時のみ scorable
- effective qualification: v11
- materialization: v23

runner completeness test は V9 を含む全 profile を coverage する。runner-freeze preparation 時点では `fixtures/evidence-relevance-holdout-v9` directory 自体を作成しないため、fresh corpus が runner wiring に影響することはない。

## Runner freeze

最初の runner freeze `engine-0.6-evidence-relevance-holdout-v9-runner-freeze` / `17ea49bc6b332bb985971b661830c80665ad5dd4` は、provider 観測前の validate-only 監査で v8 由来の `issue == 462` binding を引き継いでいることが判明した。tag は immutable evidence として保持し、移動・削除しない。この v1 runner では #468 corpus を受理できないため acceptance runner としては使用しない。

修正版は V9 のみ `issue == 468`、historical V1-V8 / development profiles は `issue == 462` を明示する。runner-freeze-v2 は commit 4674ea9a923788835d0e9bf53bdd7747d6648af8 / tag `engine-0.6-evidence-relevance-holdout-v9-runner-freeze-v2` で固定し、exact-head CI 8/8 PASS 後に push 済み。

この v2 tag の後に fresh holdout-v9 corpus を author した。corpus は holdout v1-v8 と successor-v5/v6/v7/v8/v9 development surface に対して case ID / canonical entity / task / exact signal / exact 8-token candidate-signal n-gram overlap 0 の fresh independent surface とする。corpus 自体を freeze するまでは provider observation を禁止する。


## Fresh corpus

V9 corpus は runner-freeze-v2 tag が存在した後にのみ author した。pre-v2 の破棄 draft は v2 freeze 前に削除し、commit / freeze / observe は一切していない。

candidate は26 case = Relevant 8 / Irrelevant 10 / Ambiguous 8。6 coarse relation 全てと、v11/v23 固有の semantic-frame positive、distinct-target same-relation negative、surface lookalike、explicit target/relation absence、prompt-injection inertness、identity mapping、URL ownership gap、shared ownership、truncation、single-near-sibling ambiguity、general-availability cross-frame、hard limit を意味しない exact-target numeric observation を含む。

holdout v1-v8 と successor-v5/v6/v7/v8/v9 development manifest に対し、case ID / canonical entity / task / exact signal / exact 8-token candidate-signal n-gram overlap 0 を CI で強制する。

frozen v11/v23 の offline expected observation は26/26 exact。validate-only でも issue 468、V9 configuration/suite、annotation protocol v11、fixed core v9、planned 26、completed 0、non-scorable を確認済み。

## Canonical provider gate

one-shot acceptance surface は Mistral ministral-8b-latest、Google gemini-3.5-flash-lite、Groq openai/gpt-oss-120b の required 3 provider。

全provider 26/26完走・provider failure 0を必須とする。correctness は wrong-target Relevant 0、utility は false relevance rejection / Relevant-left-Ambiguous / utility miss 0、materialization は26/26 exact、effective authority qualification は26/26 exactかつ identity / relation / scope-risk miss 0を必須とする。workflow rerun は拒否する。

## Groq admission

v9 admission anchor は immutable canonical holdout-v8 run 36946457925 の実Groq armから再計算した。

- observed tokens: 66,212
- Groq completion: 2026-10-02T02:44:50Z
- modeled starting headroom: 55,000
- inter-case delay: 300,000 ms
- minimum request interval: 10,000 ms
- 26-case arm中のmodeled refill: 約17,962.96 tokens
- conservative modeled post-v8 headroom: 約6,750.96 tokens
- required v9 starting headroom: 55,000
- self-budget: 70,000
- conservative 55K-headroom floor: 2026-10-02T08:32:14Z

floor は経過済み。material な intervening organization-level Groq usage が判明した場合は canonical freeze tag push 前に re-anchor する。

## Corpus freeze

intended corpus freeze coordinate: engine-0.6-evidence-relevance-holdout-v9-freeze。

tag push 前に exact-head CI、surface checksum、frozen successor semantics、runner-freeze-v2 invariance、offline holdout tests、validate-only、Clippy、rustfmt、YAML parse、diff check を全て green にする。

provider observation は corpus freeze tag からのみ開始する。
