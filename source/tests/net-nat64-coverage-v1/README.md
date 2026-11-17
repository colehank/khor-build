# IPv6-only NAT64 与冷连接夹具

使用现有 patchbay `IspV6` 的 SIIT＋IPv4 masquerade，不修改宿主网络或实现第二套转换器。手机 namespace 没有 IPv4 地址；Relay 实际 listener 绑定 IPv4。私有 DNS 提供真实 A 与固定 `64:ff9b::/96` 合成 AAAA。

先让 IPv6 client 的独立 UDP nonce 经转换到 IPv4 server 并原样返回，双端分别确认地址族；再验证系统 DNS 能解析合成 AAAA。随后启动原产品 CLI、通过同一域名冷配对，保持原十秒冷连接与双向两秒目录期限。原生失败只有在这些前置 oracle 全绿时才可归类为产品失败。

这是固定 DNS64 答案的模型，不是动态 DNS64 算法、464XLAT、IPv4 literal 兼容性或真实 iPhone 的证明。当前夹具尚未执行；不能提前声称产品红已复现或修复。运行边界与开销夹具相同：远端私有模板、准确 source/CLI hash、owner/资源/清理门，本机 source-only。见 `docs/net-optimization/DNS-NAT64.md`。
