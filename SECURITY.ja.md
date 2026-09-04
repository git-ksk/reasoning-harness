# セキュリティポリシー

## 脆弱性の報告

認証情報の漏えい、trusted verification boundary の迂回、または untrusted model output への権限付与につながる脆弱性については、公開 issue を作成しないでください。

利用可能な場合は、このリポジトリの GitHub private vulnerability reporting を使用してください。その経路が利用できない場合は、exploit の詳細を公開せず、GitHub プロフィール経由でリポジトリ所有者に非公開で連絡してください。

## セキュリティ境界

以下は security-sensitive な不変条件です。

- provider credentials は artifacts、logs、fixtures、model-visible prompts に決して入れてはならない。
- `ReasoningCandidate` は evidence や verification receipts を作成できない。
- claims を trusted states に昇格させたり contradiction authority を確立したりできるのは、harness-owned passes だけである。
- verification receipts は、ちょうど1つの claim と有効な harness-owned evidence に bind されなければならない。
- live-provider CI は manual/isolated のまま維持し、untrusted pull-request code に repository secrets を公開してはならない。
