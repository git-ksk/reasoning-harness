# RSD0 残余エビデンス十分性の発見

Tracking: #91, #116。

RSD0 は、採用済みの D3 runtime より意図的に狭い問いを扱う。

> D3 が現在表現できるすべての blocker を通過した後でも、提示された evidence は関連しているが、target conclusion を正当化するには不十分なケースは残るか？

新鮮な calibration-only RSD0 corpus に対する答えは **yes** である。これは control-plane の盲点に関する研究結果であり、model-backed successor がすでに安全に採用できるという主張ではない。

## データ境界

RSD0 は fresh かつ pre-observation である。

- fixture は `fixtures/evidence-sufficiency-rsd0/` の下だけに存在する。
- semantic holdout-v4/v5 の fixture や観測結果を読んだり、変換、再ラベル、corpus 導出に使ったりしない。
- label と rationale は provider-backed sufficiency run の前に commit する。
- provider/model output、score、confidence、verification receipt、authority-bearing field は RSD0 annotation contract に含めない。
- operational failure は sufficiency label ではない。

fixture validator は明示的な `EvidenceRequirement` も拒否する。これは意図的である。typed D3 requirement としてすでに表現できるケースは既存の decidability surface に属し、residual corpus には属さない。

## Label 契約

RSD0 は次の3つの diagnostic label だけを事前宣言する。

```text
sufficient
insufficient
mixed
```

解釈は次のとおり。

- `sufficient`: 選択された evidence が、harness-owned request が宣言した decision-critical information を、answerability decision を許可できる程度にカバーする。これは correctness evidence ではなく、`VerificationReceipt` や epistemic promotion を生成できない。
- `insufficient`: 関連する evidence は存在するが、1つ以上の decision-critical information needs が欠けているため、resolution/additional evidence なしに assertive answer を進めてはならない。
- `mixed`: evidence が実質的に分断されている、または単純な globally-sufficient 判定を安全に行えない程度に部分的である。product control では `insufficient` と同じ resolution/abstention 方向に保守的に適合するが、独立した research label である。

## Fresh コーパス

初期 corpus は12個の synthetic calibration case からなり、4つの workload family に各 label の case を1つずつ持つ。

| Family | Sufficient control | Insufficient residual | Mixed residual |
| --- | --- | --- | --- |
| incident root cause | incident connection + alternative separation + targeted recovery | correlated DB latency only | DB failures plus simultaneous network-path loss |
| backup / RPO | complete backup coverage + successful restore evidence | backup schedule only | one required state restored, another unresolved |
| rollout safety | representative full-window canary + error/latency guardrails | early partial observation only | error guardrail passes while latency guardrail fails |
| capacity planning | peak demand + every declared bottleneck/headroom | average demand + compute ceiling only | compute headroom good while DB bottleneck violates threshold |

case は、D3 の現在の typed blocker vocabulary に表現されていない残余 information pattern、すなわち required component 全体の completeness、observation-horizon adequacy、alternative elimination、aggregation/globality、materially mixed indicators を意図的に狙う。

## 決定論的 RSD0 の結果

`semantic_sufficiency_rsd0` はすべての artifact を validate し、変更されていない D3 decidability function で同じ target を評価する。

結果:

```text
fixtures:                     12
D3 permit:                    12 / 12
predeclared sufficient:        4 / 12
predeclared insufficient:      4 / 12
predeclared mixed:             4 / 12
non-sufficient surviving D3:   8 / 12
```

各 family に sufficient control があるため、future RSD1 gate は常に abstain するだけで表面的に安全な score を得られない。一方、事前宣言した `insufficient | mixed` の8 case はすべて D3 上 `permit` として残り、現在の typed gate を超える measurable residual gap を示す。

これは **D3 が contract に失敗したことを意味しない**。D3 の `permit` は常に「この gate が所有する deterministic blocker が見つからなかった」だけを意味する。RSD0 は、より広い task で `permit` を answerability control として有用にするには、natural-language product に追加の evidence-sufficiency coordinate が必要だと示す。

## 文献上の根拠

設計は最近の研究にある次の2つの有用な区別に従う。

- Joren et al., *Sufficient Context: A New Lens on Retrieval Augmented Generation Systems* (ICLR 2025) は、提示された context に回答に十分な情報があるかを、answer generation 自体とは別の property として扱う: <https://openreview.net/forum?id=Jjr2Odj8DJ>。
- Gu et al., *Bridging the Detection-to-Abstention Gap in Reasoning Models under Insufficient Information* (2026) は、missing information を検出するだけでは不十分で、generation が unsupported final answer に進み得る点を強調する: <https://arxiv.org/abs/2605.28070>。
- SConU (ACL 2025) は RSD0 の authority mechanism ではなく、calibrated selective uncertainty の後続 RSD3 anchor として扱う: <https://aclanthology.org/2025.acl-long.934/>。

これらの論文は research question と metrics の動機付けである。Harness authority や label を定義するものではない。

## RSD0 の判断

RSD0 は predeclared acceptance criteria を満たす。

- 4つの workload family と3つすべての label が存在する。
- すべての fixture が valid で D3-permitted である。
- `insufficient` と `mixed` の両方の residual case が D3 を通過する。
- 各 family に sufficient control がある。
- frozen holdout path は loader の外側にある。
- fixture contract に model-owned authority はない。

したがって **RSD1 は正当化される**。その frozen calibration contract は [evidence-sufficiency-rsd1.md](evidence-sufficiency-rsd1.ja.md) に記載する。次 phase では、monotone product rule を変更せず、狭い model-backed `sufficient | insufficient | mixed` coordinate を試験できる。すなわち、`sufficient` は authority を生成できず、`insufficient | mixed` は conservative resolution または abstention を維持・強制することだけができる。
