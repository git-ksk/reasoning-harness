# Cross-model live concurrency policy

Cross-model評価は **provider-aware lane** を使う。共通のdeterministic/no-network preflightでlive処理をgateしてよいが、credentialを使うlive観測開始後は、無関係なproviderを便宜上ひとつのglobal serializationへまとめない。

## Scheduling rule

1. 評価面が共通ならshared preflightを1回実行する。
2. preflight後はproviderごとの独立laneへfan-outする。具体的なshared external resourceがない限り、異なるproviderは並行実行してよい。
3. provider内のmodel-job parallelismは、quota scopeの文書化とrepositoryでの実測に基づき個別に決める。
4. model内fixture concurrencyはmodel-job parallelismと別制御とする。model jobを直列化していても、runner/providerの組み合わせで検証済みなら1 model内のfixtureを並行化できる。
5. GitHub Actions側のparallelismに関係なく、provider側のpacing、retry、`Retry-After`、token budgeting、rate-limit telemetryを優先する。
6. quota/rate-limit/availability failureはsemantic/correctness failureではなくoperational evidenceとして記録する。
7. scheduling policy更新だけを理由にfreeze済み観測を変更・rerunしない。

## Repository defaults

| Provider | Model-job default | In-model default | 理由 |
| --- | ---: | ---: | --- |
| Mistral | 1 | 1 | repository-level live concurrency groupでshared account-level limitを保護する。 |
| Google | 1 | 1 | model jobは直列。fixture concurrencyは別途検証済みのsurfaceだけ増やせ、semantic-judge系では現在2の実績がある。 |
| NVIDIA Hosted NIM | 1 | routine Nemotron Lightning runnerでは4 | 複数modelによるaccount pressure増幅を避けつつ、検証済みmodel内の遅いrequestを重ねる。 |
| Groq current Free-tier replication targets | 現行3 model jobまで | 1 | 現行3 targetはper-model quota controlを使い、各modelでrequest/token pacingとbounded retryを維持する。 |

これはrepositoryのoperational defaultであり、provider quotaの普遍的な主張ではない。並列度を上げる場合はactive plan/model scopeのevidenceが必要。下げる場合も、無関係なproviderまでglobal serializationへ落とさずprovider laneを維持する。

## Frozen historical exceptions

Issue #208 / freeze済みproduct-external-info v4 replicationも本standard以前のsurfaceで、Mistral+Googleを1つの`max-parallel: 1` matrixにしている。semantic surfaceはimmutableのまま保持する。

Issue #256 / `natural-language-e2e-v11-cross-model-v1-freeze` はGoogle+Groqをひとつのmixed matrixに入れ、`max-parallel: 1`でfreezeした。このpolicy gapを認識した時点でfirst live observationはcanonical boundaryへ入っていたため、このworkflowはhistorical exceptionとして変更せず保持し、successor replication identityから本policyを適用する。

`config/cross-model-concurrency-policy.json` がmachine-readable policyで、`scripts/validate_cross_model_concurrency_policy.py` は新規のmixed-provider global serializationをexact frozen historical exception以外では拒否し、既存Mistral/Google/Groq controlも検証する。
