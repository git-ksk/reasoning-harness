# v0.3.0 external-resolution acceptance

#179ではhistorical Stage-C / RSD2 / semantic holdoutを一切使わない、**non-frozen product acceptance lane**を追加します。

`reason-external-resolution-acceptance`は自分自身を`external_command_v1` / `trusted_command_verifier_v1` subprocessとして再起動し、実際のprocess I/O、admission、qualification/re-verification、trusted verification、budget、typed operational failure、finalizationをCIで通します。

8ケースはfresh recovery、opaque acquisition + trusted verifier recovery、stale、wrong-scope、irrelevant、conflict、transport failure、budget exhaustionです。acceptance gateはunsupported grounded claims = **0**、missed target insufficiency = **0**、false abstention = **0**、最低1件のsafe recoveryです。初回runは全条件を満たし、safe recoveryは2件でした。

network依存はrequired CIにしません。2026-09-04に`--live-aws`でpublic AWS What's New RSSを取得し、fresh provenance付き`aws.whats_new_feed_available=true`をinitial `unknown`から`accept`へ安全にrecoverしました。HTTP 200、RSS `lastBuildDate=Fri, 04 Sep 2026 03:03:01 GMT`、unsupported grounded claims 0です。machine-readable observationは`docs/observations/v0.3.0-external-resolution-live-2026-09-04.json`に保存します。
