# Reason human answer presentation

Reason CLI 0.5.0では、hidden reasoningを公開したりmodel proseをauthorityへ昇格したりせず、Harness-owned resultを通常のuserにも理解しやすく表示します。Harness Engine 0.4.2のcorrectness semanticsは変更しません。

## human outputのsection

通常のhuman natural-language pathでは次を表示します。

1. **Answer** — grounded / qualified resultでは、既にfinalize済みのexposed answerだけを表示します。unresolved / requires-verification / contradictoryではrenderer proseを使わず、Harness-ownedの固定文言を表示します。
2. **Verified facts** — final Harness artifactが`known`または`supported`としたcanonical `key=value` propositionだけを表示します。candidate/claimの自由文statementをfactual authorityとして再利用しません。
3. **Unresolved / qualified** — inferred / assumed / unknown / contradicted / uncovered、またはtyped evidence-qualification findingの影響を受けるcanonical propositionを表示します。
4. **Evidence / sources** — verified claimが参照するevidence IDだけをsource / provenance class付きで表示します。raw evidence observationはdumpしません。
5. **Untrusted context** — file/stdin/conversation contextはsourceだけを別sectionへ出し、supporting authorityではないことを明示します。
6. **Acquisition / verification notes** — typed admission rejectionやresolver failureを短いoperational wordingへ変換します。internal reasoning traceは表示しません。

footerにはfinalization status、verified/unresolved件数、factual coverage、answer-safety configuration identityを残します。

## interactive inspection

interactive turn完了後は次を使えます。

```text
/status
/evidence
```

`/status`はverified facts、unresolved/qualified item、typed acquisition note、compact statusを再表示します。`/evidence`はsupporting provenance、untrusted-context source label、acquisition noteを再表示します。どちらもmodel/resolverを呼ばず、epistemic stateを変更しません。

`reason -c` / `reason -r`直後でも、保存済みtyped `ReasoningThread` checkpointとfinalizationからviewを再構成します。presentation transcriptを別途保存する方式ではありません。

## machine output

`--format json`のmachine-oriented contractはこのpresentation layerでは変更しません。human viewは既存typed stateから派生するだけで、新しいauthority-bearing wire formatではありません。
