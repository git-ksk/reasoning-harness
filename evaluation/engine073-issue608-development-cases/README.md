# #608 0.7.3 development source scoping — initial 40 + adversarial 46

This candidate-branch-only corpus is developer-authored. It is source-level regression, not a fresh independent final-answer acceptance sample.

- initial40.json: unchanged original 20 positive / 20 negative #607 development cases.
- adversarial46.json: separately authored 10 positive / 36 negative. In an intermediate candidate, two new negatives rn10 and rn36 became false-positive accepts. Those regressions were discovered before adding exact scope guards for unowned product nonexistence and release-date hoax.
- Final candidate should classify 30/30 positives and 56/56 negatives correctly on BOTH v17 and v30 source relevance axes. Original #520 tests are still required.
- Do not treat these 86 developer-authored examples as the independent 24-case final-answer user evaluation (#531), model accuracy, citation quality or release result.
