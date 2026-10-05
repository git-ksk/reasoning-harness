# Engine 0.6 evidence relevance: v7 required-provider gate decision

Status: v7 live observation前に固定するprovider-role decision。

## Decision

calibration v7のrequired canonical armは以下の2本とする。

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Google `gemini-3.5-flash-lite` はv7の **required operational gate** から外す。semantic downgradeではなく、Googleが誤ったrelevance判断をしたという意味でもない。v4/v5/v6の完了Google observationはsemantic safe/exactを維持しており、反復した問題はcanonical run availabilityだった。

Google historical evidenceはimmutableかつ明示的に残す。

- v4: 30秒envelopeで23/26 operational
- v5: 60秒envelopeで11/26 operational、明示的HTTP 503 high-demandを含む
- recovery smoke: 4/6 operational
- v6: 1件success後に60秒timeout 2件、run-level circuitが残り23 requestを抑止

v7ではGoogle live requestを追加しない。将来Googleを再qualifyする場合は、passing calibrationへ暗黙に戻すのではなく別freezeのoperational studyで行う。

## Groq GPT-OSS 120Bを事前固定する理由

#462 semantic calibration outcomeと独立したevidenceだけで選定する。

1. repo履歴: Engine 0.5 final-v3 canonical run `35457038163` でGroq `openai/gpt-oss-120b` はvalidated-reference rowとして3/3、correctness violation 0、session replay 0でaccepted。
2. repo履歴: Engine 0.5 final-v2でも同modelはvalidated required PASS。
3. current provider status: Groq公式で `openai/gpt-oss-120b` はPreviewではなくProduction Model。
4. current capability: JSON ObjectおよびJSON Schema / Structured Outputs対応が公式に明記。
5. Harness adapter: outputはuntrusted candidateのまま、Harness-owned parser/materializer、bounded 429 retry、structured-output fallback retry、request/TPM pacing、GPT-OSS reasoning minimize controlを既に備える。

freeze前に確認した現行公式reference:

- https://console.groq.com/docs/models
- https://console.groq.com/docs/model/openai/gpt-oss-120b
- https://console.groq.com/docs/structured-outputs

NVIDIAはroutine Nemotron candidateにprotocol / timeout incompleteの既存repo evidenceがあるため採用しない。Qwen 3.8を#462 corpusで比較して勝者選択することもしない。それはprovider choiceをcalibration dataへtuneするselection biasになる。GPT-OSS 120Bは事前のvalidated-reference実績とcurrent Production designationがより強い。

## Evaluation invariants

required provider変更で以下は変えない。

- binding proposal v2
- target-first materialization v3
- strict Harness-owned identity floor
- calibration 26 caseすべて
- expected proposal/disposition labelすべて
- case単位60秒 elapsed budget
- max 2 model calls / 192 output tokens
- zero-tolerance semantic correctness gate
- operational provider failure 2件連続でfail-fastするrun-level circuit

v7結果はv7単独で判定し、v4/v5/v6を新provider setでrescoreしない。
