# Reason CLI 0.5.0 fresh-install release acceptance

**状態:** **Harness Engine 0.4.2** を変更せずに **Reason CLI 0.5.0** をrelease-readyと判定するPhase 5 / Issue #374のgateです。

このgateは、既存product contractと新しいfresh-install laneを組み合わせます。全lower-level testを巨大なsuiteへ重複コピーせず、課金・quotaを消費するlive provider callも必須にしません。

## Gate構成

必須evidenceは次のworkflowで作ります。

| Evidence lane | 確認内容 |
| --- | --- |
| `fresh-install-acceptance` | 4種類のrelease形状native archiveをbuildし、別consumer jobでRust toolchain actionを入れず、`cargo`/`rustc`を解決できないPATHから実行します。CLI/Engine identity、empty-home setup、明示provider/model選択、config確認、local-only doctor、typed one-shot recovery、JSON互換性、lifecycle dry-run/fail-closed、Unix private permission、secretの再帰scanを確認します。live provider requestは0回です。 |
| `installer-smoke` | supported platform上のproduction `install.sh` / `install.ps1` contract。split-release installer contractではprovenance/checksum/version/platform異常をfail closedで拒否します。 |
| `cli-platform-smoke` | Linux/macOS/Windowsのproduct CLI互換、MCP surface、JSON/non-interactive regression、subprocess environment isolation。 |
| `credential-store-smoke` | macOS/Windows native credential store round tripとscoped MCP OAuth credential lifecycle。Linux headless/unavailableはdeterministic testでtyped failureとなりplaintextへfallbackしません。 |
| `lifecycle-smoke` | 全supported platformでupdate/rollback/uninstall contract。 |
| `ci` | `cargo fmt --check`、workspace Clippy `-D warnings`、full workspace test、deterministic CLI/fixture regression、変更していないEngine correctness suite。 |

Phase 5ではpublic `reason-v0.5.0` tag / GitHub Releaseを作りません。公開操作はこのgateがgreenになった**後**だけです。そのためpre-tag gateから、まだ存在してはいけないrelease URLをdownloadすることはできません。代わりにcandidate workflowがproduction releaseと同じarchive layoutを作り、production installer/updaterのprovenance policyはdeterministic contractで検証し、実公開時のGitHub/Sigstore attestationはrelease workflowが担当します。

## Acceptance matrix

| 要件 | Automated evidence |
| --- | --- |
| macOS/Linux/Windows fresh install | `fresh-install-acceptance` 4-platform producer/consumer matrix |
| Rustなしnative binary導入 | consumer jobはRust actionなし、`cargo`/`rustc`を解決できないPATHで実行 |
| CLI/Engine identity分離 | packaged `reason --version` と `reason doctor` でCLI `0.5.0` / Engine `0.4.2`を必須化 |
| untrusted projectからexecutable/MCP/verifierがsilent activationしない | full `ci`の`project_trust` integration test + platform product smoke |
| secretをargv/history/plaintextへ出さないcredential setup | fresh-installのdocumented env path、auth/setup contract、native credential-store smoke、secret再帰scan |
| explicit provider/model selection・silent fallback禁止 | fresh-install model switch/restore + model lifecycle contract |
| one-shot natural-language path | packaged binaryでconfigured-default natural command pathとtyped credential recoveryを実行。provider call不要のdeterministic natural semanticsはworkspace testで維持 |
| interactive + follow-up | TTY entry、follow-up、context追加、status/evidence/usage、persistence boundaryをdeterministic interactive testで確認 |
| persisted create/continue/resume・crash/concurrency・identity pin | managed-session / interactive identity pin / product session contract |
| `reason doctor` local / optional live boundary | packaged local-only doctor + deterministic doctor network/live-check tests |
| actionable failure recovery | packaged credential failure + Phase 4 remediation contract |
| credential/provider/quota/config/trust/MCP/session/network/update/distribution recovery | Phase 4 remediation catalog、doctor、lifecycle、MCP、installer/provenance contract |
| project trust | `project_trust` integration suite |
| MCP/resolver/verifier secret isolation | `cli-platform-smoke` sentinel + provider isolation tests |
| usage/resource limits | `usage_budget` pre-call / exhaustion / JSON tests |
| privacy / purge / ephemeral | `local_privacy`、interactive ephemeral、managed-session purge、uninstall retain/purge contract |
| update provenance / Engine change確認 | lifecycle + release provenance contract。Engine identityは独立表示 |
| rollback | lifecycle contract + packaged historical-boundary fail-closed smoke |
| uninstall | packaged dry-run + cross-platform lifecycle smoke。明示purgeしない限りdata/credential保持 |
| JSON/non-interactive互換 | packaged `run`/`verify` + `cli-platform-smoke` + `ci` |
| proxy/custom CA/headless | Phase 4 doctor/network、reqwest proxy、custom CA fail-closed、Linux headless credential test |
| zero secret leakage | stdout/stderr、fresh-home再帰scan、auth/setup/doctor/MCP、subprocess sentinel isolation |
| artifact/distribution tamper resistance | installer checksum/provenance rejection、signed manifest contract、release workflow GitHub/Sigstore attestation policy |
| Engine 0.4.2 semantics不変 | package座標assert（CLI=0.5.0 / Engine=0.4.2）+ full workspace/fixture regression。protected eval/holdoutはPhase 5で変更しない |

## Manual / physical status

Issue #374に物理端末やGUI専用acceptanceはありません。supported release platformはnative GitHub-hosted runnerで自動化できます。public release publicationは意図的に**gate通過後**であり、manual acceptance blockerではありません。

## Release boundary

Phase 5をgreenにできるのは、Phase 5 PR上とmerge後`main`の両方で上記required checkが通った場合だけです。実装変更によってrequired workflowがskipされた場合、P0 issueが残る場合、product acceptance都合でprotected eval/holdoutを変更した場合はCOMPLETE扱いにしません。
