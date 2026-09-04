# v0.3.0 external-resolution の受け入れ

Issue #179では、意図的に**non-frozen product acceptance lane**を追加します。historical Stage-C、RSD2、semantic holdout、その他の観測済みresearch corpusを読み取り、変更し、relabelし、tuneするものではありません。

`reason-external-resolution-acceptance`は、実際のsubprocess adapter boundaryを通して8つのdeterministic workloadを実行します。binary自身が`external_command_v1`として、必要な場合は`trusted_command_verifier_v1`として再起動します。これによりCIをcredential-freeに保ちながら、process I/O、admission、qualification/re-verification、trusted verification、budget、typed operational failure、finalizationを検証します。

宣言されたCI setは次のとおりです。

- freshなexact external fact -> safe recovery
- opaque acquired data -> independent trusted verifier -> safe recovery
- stale evidence -> `unknown`
- wrong-scope evidence -> `unknown`
- freshだがirrelevantなdata -> `unknown`
- admitted conflicting evidence -> `reject`
- transport failure -> semantic `unknown`を伴うtyped operational terminal
- budget exhaustion -> semantic `unknown`を伴う`exhausted`

gateでは、unsupported grounded claims = **0**、missed target insufficiency = **0**、false abstentions = **0**を要求します。また、expected-unknown targetはすべてungroundedのまま残り、initially-unsupported -> verified recoveryが少なくとも1件ある必要があります。初回accepted runは期待された結果8/8、recovery 2件、acquisition success 6件、独立したtrusted-verifier success 1件、mean final-claim coverage 1.0でした。

## Live public-information のスモークテスト

network依存のevidenceは、意図的にrequired CI dependencyではありません。2026-09-04、同じbinaryを`--live-aws`付きでpublic AWS What's New RSS feedに対して実行しました。resolver subprocessがHTTP requestを実行し、freshなacquisition timestamp付きでsource `aws:whats-new:feed`を出力し、Harnessがexact typed fact `aws.whats_new_feed_available=true`をadmit/re-verifyしました。

記録結果は、HTTP 200、RSS `lastBuildDate=Fri, 04 Sep 2026 03:03:01 GMT`、initial verdict `unknown`、final verdict `accept`、external call 1回、unsupported grounded claims `0`です。machine-readable observationは[`v0.3.0-external-resolution-live-2026-09-04.json`](observations/v0.3.0-external-resolution-live-2026-09-04.json)にcommitされています。

このsmokeが示すのは、current/public provenanceとfreshness handlingです。これはobservationであり、frozen benchmarkでも、AWS endpointが今後も利用可能であり続けるという恒久的な主張でもありません。
