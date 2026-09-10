# Operational observability と conservative paired bounds — metric v13

metric v13 は、v29後の fresh successor にだけ適用する **prospective な release evaluation identity** である。frozen v26 / v28 / v29 の rerun・rescore・reclassify は行わない。

## 問題

released control が途中で operational terminal になっても、それ以前に確定済みの outcome は存在しうる。terminal 後に evaluator が埋めた `false` / `0` をすべて semantic failure と扱うと candidate に不当な得点を与える。一方、incomplete case を丸ごと除外する complete-case deletion も、operational missingness が random とは限らないため採用しない。

v13 は各 metric を `observed` / `censored` / `not_applicable` に分ける。後から operational terminal が起きても positive monotone witness は `observed` のまま保持する。通常完了まで到達しないと否定できない outcome は `censored` とする。`not_applicable` は意味上の非適用であり欠損ではない。

## Metric frontier

| Metric | 後続terminalがあっても observed | operational terminal で censored | N/A |
| --- | --- | --- | --- |
| target recall | expected target membership を実際に観測済み | target がまだ観測されていない | investigation以外 |
| tool selection | qualifying relevant capability の実行を観測済み | qualifying execution をまだ観測していない | investigation以外 |
| false abstention | grounding 到達を観測し abstention ではないことが確定 | semantic finalization がterminalで中断 | field自体が非適用 |
| avoidable follow-up stall | 1 action以上実行され `action_count != 0` が確定 | 0 action のままterminal | follow-up以外 |
| trigger exposure | configured cache の typed `no_result` を観測 | trigger不存在を確定する前にterminal | follow-up以外 |
| continuation eligibility | trigger観測済みで、そのtrigger時点のeligibilityが確定 | trigger/eligibility前提がcensored | trigger非露出が確定 |
| mechanism conformance | eligible continuation のconformance、または具体的な誤follow-upを観測 | eligible continuation outcome前にterminal | continuation非eligible |
| downstream utility | conformant follow-up後のoutcomeを観測 | post-follow-up outcome前にterminal | mechanism非conformant/非適用 |
| correctness violation | violationを実際に観測 | incomplete controlでのzeroは完全なclean証明に使わない | — |

candidate の operational failure は従来どおり release hard failure。candidate correctness violation も独立hard failureである。authority / admission / verification / finalization / identity / session / MCP authority の境界は変更しない。

## Conservative bounds

fixed-denominator の binary/count metric は、observedを確定値、censoredを全許容binary rangeとして扱う。conditional rate は、case-level observability と semantic prerequisite に整合する numerator/denominator の全割当を列挙して上下限を求める。complete-case deletionもpoint imputationも行わない。

candidate exact value `C`、control interval `[L, U]` のとき:

- higher-is-better non-regression: `C >= U` のときだけPASS、strict improvement: `C > U` のときだけPASS;
- lower-is-better non-regression: `C <= L` のときだけPASS、strict improvement: `C < L` のときだけPASS;
- required claimを証明できない重なりは `INCONCLUSIVE`;
- `INCONCLUSIVE` はrelease不可。

## Historical offline characterization

以下は **diagnostic characterizationのみ**。frozen canonical outcomeは変更しない。

| Historical control | Operational failures | Target recall | Tool selection | False abstentions | Avoidable stalls | Trigger count |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| v26 Gemini | 1 | `[1.0, 1.0]` | `[0.6, 0.7]` | `[5, 6]` | `[2, 3]` | `[0, 1]` |
| v28 Gemini | 2 | `[1.0, 1.0]` | `[0.6, 0.8]` | `[4, 6]` | `[2, 3]` | `[0, 1]` |
| v28 Gemma | 1 | `[0.8, 0.8]` | `[0.8, 0.9]` | `[4, 5]` | `[0, 1]` | `[2, 3]` |
| v29 Gemini | 2 | `[1.0, 1.0]` | `[0.6, 0.8]` | `[5, 7]` | `[2, 3]` | `[0, 1]` |

狙いどおり、繰り返し発生したGemini terminal caseではtarget recallはterminal以前に確定しているためexactのまま残り、後段のnegative outcomeだけがuncertainty intervalになる。

sanity checkとしてv13-style boundsをimmutable v29 Gemini pairへ仮適用しても、v29はPASSにはならない。target recall / tool selection / avoidable stalls / trigger exposureは保守的にnon-worse/improvementを証明できる一方、false abstentionsは candidate `7` に対し control `[5, 7]` なのでrequired non-regressionは `INCONCLUSIVE` になる。frozen v29はv12で確定した元の `VALID RELEASE FAIL` のままである。

## v30 integration boundary

fresh v30は provider credential を一度でも使う前に `config/natural-language-e2e-metric-v13.json` をlockする。operational terminal reportはcase kindとcoverage contractを含むsemantic envelopeを保持しなければならず、欠けている場合observability evaluatorはfail-closedする。scoringにはcontrol/candidate双方に共通するcanonical report telemetryだけを使い、candidate-only structured-generation diagnosticsは `scoring_input=false` のままとする。Mistralはpaired、Groqは別途変更を正当化しない限りcandidate-onlyを維持する。Gemini/Gemmaは別Google jobとして `max-parallel: 2` で実行可能だが、各model row内部のcontrol→candidate canonical disciplineは維持する。

Actions logへのprogress heartbeatはraw evidenceを変更しない範囲で許可する。arm単位timeoutを入れる場合はcontrol/candidateで事前に同一値を固定し、retryを追加せず、timeoutをoperational failureとして保存し、可能な限り反対armのcanonical evidenceも保存する。
