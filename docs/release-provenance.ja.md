# Release provenanceとinstaller trust

日本語 | [English](release-provenance.md)

**Status:** split後のReason CLI release（`reason-v0.5.0`以降）では必須です。historical unified releaseの`v0.4.2`以前はこのcontractより前に公開されており、immutableのまま維持します。

Reasonは、同じRelease pageから取得したchecksumだけを唯一のtrust rootにはしません。split CLI releaseでは **GitHub OIDC / Sigstoreに基づくkeyless GitHub artifact attestation** を主たるprovenanceとし、SHA-256をdefense in depthとして併用します。

## 何を検証するか

native archiveごとにrelease workflowがGitHub attestationを発行します。supported installerは`reason-v*` archiveについて、`gh attestation verify`で次をすべて確認できた場合だけ受け入れます。

- repository: `git-ksk/reasoning-harness`;
- signer workflow: `git-ksk/reasoning-harness/.github/workflows/release-cli.yml`;
- source ref: 要求したexact `refs/tags/reason-vX.Y.Z`;
- self-hosted runner由来のprovenanceではないこと;
- attested subject digestとdownloadしたarchiveが一致すること。

provenance検証後にも`SHA256SUMS`を照合し、さらにarchive内の`reason --version`が要求versionと一致してから既存binaryを置換します。

そのため`reason-v*`のinstallにはGitHub CLI **2.93.0以上**が必要です。古いverifierはfail closedします。またambientなenterprise host設定でprovenance lookupが別hostへ向かないよう、attestation検証時の`GH_HOST`は`github.com`へ固定します。

## Release manifest

split releaseにはattested `release-manifest.json`（`reason-release-manifest-v1`）も含め、次をbindingします。

- repository / signer workflow;
- exact release tag;
- Reason CLI version;
- Harness Engine version;
- Git commit;
- native archive、installer/checksum metadataと各SHA-256 digest。

これはupdate / rollback時にmutation前のCLI / Engine identity比較を可能にし、channel混同やmetadata mismatchを拒否するためのものです。manifest自体も同じrelease workflowからattestします。

## key / trust-root rotation

このprovenance layerでは長期project signing private keyを保持しません。GitHub Actionsがshort-lived OIDC identityを取得し、GitHub / Sigstore infrastructureを通してattestationを記録します。repositoryやCI secretへ固定release private keyを置く必要がありません。

verifier / trust-root rotationはsupported GitHub CLI / Sigstore trust metadataへ委ねます。Reason側では永久leaf certificateやdeveloper keyではなく、repository、workflow、source ref、runner policyを固定します。provenance serviceが利用できない、またはverificationに失敗した場合はinstall/updateをfail closedし、SHA-256だけで新しいsplit releaseを許可しません。

## Immutable releases

repositoryのimmutable releasesを有効化しています。これはfuture releaseへ適用され、公開後のrelease asset / tag変更をGitHub側でも防ぎます。attestation検証は引き続き必須で、immutability単独をprovenanceの代替にはしません。

## Historical release

`v0.1.0`〜`v0.4.2`はこのprovenance contract以前のreleaseです。既存SHA-256経路のままimmutableに維持し、historical tagへ後付けattestationは行いません。`reason-v*`と同等のtrust contractであるとも表示しません。

## macOS notarization / Windows code signing

OS-native signingはsource provenanceとは別に評価しました。現repositoryにはApple Developer ID / notarization credential、Windows code-signing identityがprovisionされていません。そのためplatform signing private keyをrepositoryへ埋め込まず、Gatekeeper / SmartScreenなどOS security controlを無効化する手順も案内しません。

externally managedなsigning identityが用意された場合は、同じartifactへnative signing / notarizationを後から重ねられます。その場合も上記repository / workflow / tag provenance checkを置き換えたり弱めたりしません。
