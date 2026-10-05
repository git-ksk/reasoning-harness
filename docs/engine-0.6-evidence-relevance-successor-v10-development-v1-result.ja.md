# Engine 0.6 evidence relevance successor-v10 development v1 result

Status: immutable development FAIL。

- Freeze tag: engine-0.6-evidence-relevance-successor-v10-development-v1-freeze
- Freeze commit: 7c21f3e3d207da72027924114009795b7508e167
- Run: 37097092197
- Candidate: effective qualification v13 / materialization v25
- Surface: 16 case x 3 matched trials/provider
- Result: FAIL

v1 tag と observation は historical development evidence として保持する。tag を移動せず、PASSへ再解釈しない。

## Mistral result

Mistral / ministral-8b-latest は48 observationsを operational には全完走した。

- successful observations: 48/48
- provider failures: 0
- confirmation matches: 21/48
- false confirmations: 3
- missed confirmations: 24

expected-confirm 8 family は全て3 trialで not_confirmed となり、missed confirmation 24件。

prompt-injection negative control は3 trial全て confirmed_different_relation となり、false confirmation 3件。

したがって専用model verifierは utility要件と standalone one-sided classification要件の両方を満たさない。

## Cancellation

Mistral完走時点で all-provider gate のPASSが不可能になったため、不要なGroq quota消費を避ける目的で run 37097092197 を意図的にcancelした。

Google / Groq は実行途中でcancel。これらのincomplete armは semantic acceptance evidence として扱わない。Google attempt telemetry と部分的なGroq rate telemetryはGitHub run artifactに残る。

final gate は正しくFAIL。

## Adjudication

この結果は v11/v23 safety regression を示さない。v11/v23 は unchanged で、v13 が DifferentRelation を生成する前には deterministic Harness cue がmodel verifierとは別に必要。

特にMistralのprompt-injection false confirmation単独では、当該controlにHarness-owned non-requested-relation cueが無いため v25 Irrelevant には到達しない。

一方、positive development familyを全てverifierがmissしており、残存over-abstentionを回復するというcandidateのutility目的を満たせない。

## Next direction

観測済みv1結果はtune/rewriteしない。

次candidateでは、exact-target / risk-none / non-positive relationに限定した狭いHarness-owned semantic cue自体をnegative-relation authorityとして扱えるかを検証し、model confirmation voteへの依存を外す。

deterministicに次を証明する:
- true requested-relation evidenceをdowngradeしない;
- generic / explicit absenceをDifferentRelationへ変換しない;
- context-gap / truncated evidenceはAmbiguous維持;
- prompt injectionはinert;
- fresh negative controlsはadversarial confirmation signalがあっても不変;
- historical relation controlsをregressionさせない。

その後、新holdout前にfull proposal + qualification + materialization pathをfresh developmentで複数provider観測する。
