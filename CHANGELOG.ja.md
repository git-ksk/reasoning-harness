# 変更履歴

`reason` CLI に関する、製品向けの注目すべき変更を記録する。research-only binary と fixture study の変更は research note に記載する。実行可能ファイルは semantic versioning に従うが、v0.x では product interface を引き続き hardening 中である。machine-readable contract identity は executable version より厳密な compatibility boundary である。

## [Unreleased]

## [0.3.0] - 2026-09-04

第3の external-preview capability release。v0.3.0 は research generation、semantic runtime、answer-safety identity を変更せず、bounded external evidence と resolution を追加した。

### 追加

- Harness-owned の source、freshness、scope、authority policy を fail-closed で扱う `external_command_v1` と `external_evidence_admission_v1`。
- external-resolution budget、typed operational failure、telemetry、replay-safe record。
- read-only `mcp_readonly_v1`、別系統の `trusted_command_verifier_v1`、任意の Rust-only `reason-mcp` native-runtime delegation。
- `external-resolution-acceptance-v1`。unsupported grounded claims と missed target insufficiency は `0`、safe recovery は2件、live AWS RSS の別系統 smoke は `Unknown -> Accept` だった。

### 維持したもの

- Frozen Stage-C/RSD2/historical holdout は変更しない。
- Semantic runtime は `semantic-decidability-d3-v1`、answer safety は `verified-target-answer-gate-v1` のまま。MCP は correctness boundary の外側である。

## [0.2.0] - 2026-09-04

native Reasoning Harness CLI の第2の external-preview release。既存の research/authority foundation 上の **product capability release** であり、frozen Stage-C/RSD2 evidence の rewrite ではない。

### 自然言語AI CLI

- 現行 `reason-natural-output-v2` JSON identity を使う直接の `reason "TASK"` AI execution を追加し、v0.1 structured product command を維持。
- provenance-aware `--file` と piped-stdin の untrusted context、bounded input size を追加。
- `--fact`、`--hypothesis`、bounded `--resolver-fact` を追加。ただし arbitrary prose/model output は trusted evidence に self-promote できない。
- final-claim coverage の後段に model-backed final rendering を追加し、renderer が exact authorized target を省略/弱化した場合の deterministic recovery を追加。
- structurally isolated な verified target の target-local qualified recovery を追加し、global `Reject`/`Unknown` と authority check を維持。

### プロダクト評価と信頼性

- incident-analysis と architecture-review workload で raw / Harness baseline / current-safety を比較する `reason-product-dogfood`。
- Google/Gemini の一時的 429、HTTP 500/502/503/504、isolated empty-model-text anomaly の bounded retry。credential、quota、deterministic 4xx/protocol、transport、timeout は fail-fast のまま。
- adapter と structured-output fallback call の provider HTTP-attempt telemetry。
- `reason-product-dogfood-v10` の exact-identity checkpoint/resume。完全完了 case だけ再利用し、active case は先頭から再開、provider/protocol failure は semantic abstention ではなく operational evidence として保持。
- Ministral 8B の6-case product revalidation で Harness target coverage は historical 0.25 slice から 1.00 へ改善。unsupported grounded claims と missed target insufficiency はともに zero。

### CLI互換性と配布

- 実際の `reason` binary を実行し、`reason-cli-output-v1`、schema ID、stdin behavior、epistemic `unknown` の exit 0、typed operational failure の exit 1、CLI usage failure の exit 2 を固定する process-level compatibility test。
- Linux x86_64、macOS arm64、macOS x86_64、Windows x86_64 で compatibility contract を実行。
- documented v1.0 readiness gate が current main で満たされても、v0.x は external-preview として明示。release automation は 0.x GitHub Release を自動で prerelease 化。

### 研究と権威情報の来歴

- frozen Stage-C candidate/holdout と historical RSD2 outcome を変更せず、prior provider failure を semantic success と解釈しない。
- successor semantic candidate は `993874fa0051d06a02c8db8f7a220a2ac7773c17`、semantic runtime は `semantic-decidability-d3-v1`、answer-safety は `verified-target-answer-gate-v1`。
- model output、retrieval prose、retry success、checkpoint reuse は verification authority の外側である。

## [0.1.0] - 2026-09-01

native Reasoning Harness CLI の最初の external preview。

### プロダクトCLI

- `reason run`、`reason verify`、`reason semantic-check`、`reason schema` を supported product surface として追加。`eval*` は research/evaluation のまま。
- non-interactive JSON input 用の stdin (`-`) と one-consumer protection。
- `reason-cli-output-v1`、`reasoning-artifact-v1`、`reasoning-candidate-v1`、`reason-config-v1`、`semantic-check-input-v1` の machine-readable contract identity/schema discovery。
- CLI flags > explicit config > project config > user config > defaults の layered non-secret config と `--no-config` hermetic execution。
- machine-readable product failure envelope と normalized provider/input/config/harness failure class。process failure は exit 1、`accept | reject | unknown` 成功結果は exit 0。

### セマンティックランタイム

- `reason semantic-check` から `semantic-decidability-d3-v1` runtime を公開。ただし soft diagnostic に final-verdict authority は与えない。
- `soft-semantic-v3` rollback selection を維持。
- semantic decision と分離した typed operational failure output。
- Ministral 8B と Google-hosted Gemma 4 31B の D3/v3 rollback live product smoke に成功。

### 配布

- Linux x86_64、macOS arm64、macOS x86_64、Windows x86_64 の credential-free product smoke。
- single supported `reason` binary の `cargo install --git` installation。
- tag-driven standalone GitHub Release archive と SHA-256 checksum。research binary は release artifact ではない。
