# Proxy / custom CA / headless network環境

**状態:** Reason CLI 0.5.0開発ラインで実装済み。Harness Engine 0.4.2のreasoning / authority semanticsは変更しません。

ReasonのHTTP clientはreqwest標準のsystem proxy挙動を使います。`HTTP_PROXY` / `http_proxy`、`HTTPS_PROXY` / `https_proxy`、`ALL_PROXY` / `all_proxy`、`NO_PROXY` / `no_proxy`はprovider adapter、remote MCP、MCP OAuth discovery/token exchange、CLI release/update通信で有効です。diagnostic outputには各変数のpresenceだけを出し、proxy URL、user/password、bypass listの値は表示しません。

## Custom CA bundle

corporate TLS interception proxyやprivate CAを通常のtrust rootへ**追加**して信頼する必要がある場合は、PEM certificate bundleへのpathを`REASON_CA_BUNDLE`へ設定します。built-in certificate verificationは維持されます。空path、regular file以外、空/過大bundle、invalid PEMはrejectします。現在の上限は1 MiBです。

TLS verificationを無効化する、invalid certificateを許可する、certificate checkをglobalに弱めるproduct optionは意図的に提供しません。recoveryではproxy、DNS、certificate chain、system trust、または`REASON_CA_BUNDLE`を修正します。

## Doctor diagnostics

`reason doctor --format json`はnoninteractive / headlessでも利用でき、`network` objectへproxy presence boolean、custom CA status、live probe statusを出します。default doctorはlocal-onlyです。`reason doctor --live-check`では、provider/auth readiness requestより前にconfigured provider hostへのbounded HTTPS connectivity probeを行います。HTTP responseが返ればtransport pathは成立とし、provider authentication/service statusは別に診断します。

network failureはprovider outage / credential failureと分離したoperational failureとしてtypedに扱います。

- `network_dns`
- `network_proxy`
- `network_tls_certificate`
- `network_connectivity`
- `network_timeout`
- `network_transport`
- `network_custom_ca`

これらはsemantic `unknown`へ変換せず、evidence admission / verification / finalizationも変更しません。
