# 外部 resolver アダプター

Reasoning Harness v0.3.0 は、外部取得を正しさのコアの外側に置く。resolver はデータを取得または計算できるが、正常終了したというだけで権威を得ることはない。

Issue #174 は、最初にサポートする外部取得アダプター `external_command_v1` を追加する。既存の `ResolutionResolver` 契約を実装し、明示的に設定された1つの実行ファイルを起動して、stdio 上で1回の JSON request/response を交換する。`reason` と設定済みプログラムの間に shell は挿入しない。すべての `--resolver-arg` / 設定引数は文字通り渡す。

アダプターは既存の制御経路内にとどまる。

```text
ResolutionRequest
  -> external_command_v1
  -> AcquiredEvidence | CandidateRevision | NoResult | HumanReviewRequired
  -> EvidenceAdmissionPolicy
  -> 通常の validation / verification / diagnostics / decision / finalization
```

## CLI と設定

自然言語経路では、1つの外部 command resolver を受け付ける。

```bash
reason "Determine the failover region" \
  --hypothesis service.failover_region=eu-west-1 \
  --resolver-command /path/to/resolver \
  --resolver-arg --json \
  --max-resolution-attempts 1
```

対応する `reason-config-v1` の断片は次のとおりである。

```json
{
  "schema_version": "reason-config-v1",
  "resolution": {
    "external_command": {
      "program": "/path/to/resolver",
      "args": ["--json"],
      "admission": {
        "authority_ranks": {"secondary": 10, "primary": 20},
        "minimum_authority_class": "primary",
        "required_scope": {
          "region": {"kind": "values", "values": ["eu-west-1"]}
        },
        "sources": {
          "example:external-source": {
            "authority_class": "primary",
            "max_age_seconds": 300,
            "scope": {
              "region": {"kind": "values", "values": ["eu-west-1", "eu-west-2"]}
            }
          }
        }
      }
    }
  }
}
```

`--resolver-command` は設定された `resolution.external_command` より CLI で優先される。`--resolver-arg` は明示的な `--resolver-command` と併用した場合だけ有効である。外部 command 経路と `--resolver-fact` は、この最初の product lane では意図的に相互排他的である。現在の bounded runtime は resolver class ごとに1つの resolver を選択するためである。

環境変数は通常の OS の意味で子プロセスへ継承されるため、統合側は環境から資格情報を取得できる。ハーネスは資格情報を `ResolutionRequest`、取得済み evidence、trusted metadata、receipt、最終出力へコピーしない。adapter/admission config は resolution telemetry 上では安定した SHA-256 由来の identity だけで表現し、文字通りの command 引数を config identity string として出力しない。

## Stdio プロトコル

`reason` は schema `reason-external-resolver-request-v1` を使い、UTF-8 JSON object を resolver の stdin に1つ書き込む。

```json
{
  "schema_version": "reason-external-resolver-request-v1",
  "adapter_id": "external_command_v1",
  "attempt_index": 0,
  "request": {
    "id": "resolution:proposition:service.failover_region=eu-west-1",
    "reason": "missing_support",
    "target": {
      "kind": "proposition",
      "proposition": {
        "key": "service.failover_region",
        "value": "eu-west-1"
      }
    },
    "resolver_class": "evidence_acquisition",
    "budget": {}
  }
}
```

resolver は schema `reason-external-resolver-response-v1` を使い、stdout に JSON response を正確に1つ書き込む。取得形式は次のとおりである。

```json
{
  "schema_version": "reason-external-resolver-response-v1",
  "contribution": {
    "kind": "acquired_evidence",
    "evidence": [
      {
        "id": "resolver-result-1",
        "source": "example:external-source",
        "observation": "service.failover_region=eu-west-1",
        "facts": {
          "service.failover_region": "eu-west-1"
        },
        "acquisition_metadata": {
          "observed_at_unix_seconds": 1788487200,
          "retrieved_at_unix_seconds": 1788487210,
          "scope": {
            "region": {"kind": "values", "values": ["eu-west-1"]}
          },
          "claimed_authority_class": "primary"
        }
      }
    ]
  },
  "cost": {
    "added_tokens": 0,
    "elapsed_ms": 0
  }
}
```

その他に許可される contribution kind は `candidate_revision`、`no_result`、`human_review_required` である。これらはすでに `ResolutionResolverContribution` に存在する。

response の wire format は意図的に閉じている。resolver response に `EvidenceMetadata`、verification receipt、verdict、最終 prose を含めることはできない。不明な response field は権威として暗黙に解釈せず、malformed output として拒否する。

## #175 後の Trust 挙動

外部 command による取得は引き続き **デフォルトでは untrusted** である。`resolution.external_command.admission` がない場合、CLI は `external_command_v1` を `RejectAllEvidenceAdmission` と組み合わせる。そのため、外部呼び出しが成功しても unsupported target を `Supported` に変えることはできない。

admission を設定した場合、resolver が報告できるのは正規化された **acquisition metadata** だけである。すなわち、正確な `source` identity、観測時刻、取得時刻、適用可能性 scope、claimed authority class である。これらは `EvidenceMetadata` ではない。`external_evidence_admission_v1` は `Evidence` が作成される前に、Harness 所有の config と照合する。

- `source` は allowlist の source entry と完全一致しなければならない
- `observed_at_unix_seconds` と `retrieved_at_unix_seconds` は必須で、取得が観測より前であってはならず、freshness は source ごとに設定された `max_age_seconds` 以内でなければならない
- 取得 scope は source に設定された最大値を超えてはならず、request/config が要求する scope をカバーしなければならない
- resolver の `claimed_authority_class` は、Harness config がその source に割り当てた authority class と一致しなければならない
- source に割り当てられた authority は、Harness 所有の `authority_ranks` における最も強い設定済み/要求済み minimum を満たさなければならない

したがって resolver の authority claim は、検査対象の assertion にすぎない。trusted `EvidenceMetadata.provenance_class` は Harness 所有の source policy からコピーされ、resolver output から昇格されることはない。欠落、stale、future、wrong-scope、unknown-authority、insufficient-authority、mismatched-authority のデータは fail closed する。

admission の拒否は `ResolutionAttempt.admission_rejection` で機械的に観測できる（例: `stale`、`scope_mismatch`、`authority_claim_mismatch`）。拒否は resolution observation のままであり、semantic evidence ではない。

admitted evidence については、Harness は設定済みの authority policy と qualification requirements も通常の input に組み込み、validation、evidence qualification、verification、diagnostics、decision、finalization を再実行する。決定論的な end-to-end regression は、fresh/in-scope/allowed な外部 evidence が、初期 unknown target をこの再検証後にのみ回復できることを証明する。一方 stale evidence は verification receipt なしのまま `unknown` である。

candidate revision は引き続き権威を得ない。改訂された candidate は、Harness 所有の evidence と policy に対して同じ通常 pipeline を単に再実行する。

## Failure と budget の扱い

アダプターはプロセス単位の wall-clock timeout と、受け入れる stdout の最大サイズを強制する。デフォルトは 30,000 ms と 1 MiB であり、`--resolver-timeout-ms` / `--resolver-max-response-bytes` または `resolution.external_command` の config field で短縮できる。値が zero の operational limit は fail closed する。

Operational failure class は明示的である。`authentication`、`permission_denied`、`policy_denied` は `denied` として終了し、`timeout` は `timed_out`、`transport` と `protocol` は `operational_failure` として終了する。executable-not-found は `unavailable` のままである。Legacy の `malformed_output`/`failed` は、凍結された過去の resolution fixture に対して retry/exhaustion 互換性を保つために残る。これらの終端状態は semantic verdict に付随するのであり、置き換えたり格上げしたりするものではない。したがって semantic `unknown` と不完全な外部 operation を区別できる。

`ResolutionCost` / `ResolutionUsage` は実際の call count、経過ミリ秒、提供された場合の added tokens、提供された場合の optional micro-USD cost を記録する。`ResolutionAttempt` は安定した adapter config identity と admission-policy identity も記録する。これらの identity は Harness 所有 config の hash であり、raw credentials や argument string ではない。`external_command_v1` に generic automatic retry は意図的にない。決定論的な authorization、policy、protocol failure を再実行しないためである。

既存の `GroundedResolutionPolicy` は resolver-class allowlisting と run/request ごとの attempt/token/time accounting の所有者であり、#178 は2つ目の correctness または budget system を作らない。ReasoningThread checkpoint は型付き attempt と identity を保持し、決定論的 replay は外部アダプターを再度呼び出さずにそれらの記録を再構成する。

## 参照スモークパス

決定論的な adapter test は一時 executable を起動し、stdin で実際の型付き request を送り、stdout で acquired evidence を受け取り、trusted metadata や receipt を紛れ込ませようとする試みが schema parsing に失敗することを別途検証する。この smoke path は network service を必要とせず、凍結された research fixture を変更せずに、実際の process I/O を試験する。

ライブ統合では、設定された executable 自体が web API、database、compiler/test tool、その他の read-only source を呼び出してもよい。返された data は依然として、上記の contribution type としてのみ Reasoning Harness に入る。MCP 固有の取得は別途 `mcp_readonly_v1` として #176 の下で実装され、`external_command_v1` に特例として組み込まれない。[Read-only MCP resolver](mcp-resolver.ja.md) を参照。
