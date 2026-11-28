# IPv6-only 地址变化功能验证

更新于 2026-11-14T04:59:07.913354+00:00。固定15场五类各3次，630/630实际逻辑读取成功，双向首业务均在原2秒门内；最慢1705510μs。此处只接受所列纯IPv6物理功能合同，完整N4/N3/N5仍未完成。

| 场 | 变化 | run | 双向首次完成 μs | 原结果 |
| --- | --- | --- | --- | --- |
| 1 | one_iid | turing-20261114T042127Z-54737e59 | 758939 / 745392 | 42/42 ok |
| 2 | one_prefix | turing-20261114T042901Z-f74a8df0 | 598302 / 597705 | 42/42 ok |
| 3 | both_iid | turing-20261114T043035Z-2643f42e | 944293 / 949209 | 42/42 ok |
| 4 | both_prefix | turing-20261114T043240Z-ce27a48e | 509911 / 609888 | 42/42 ok |
| 5 | both_prefix_relay_only | turing-20261114T044303Z-7495158c | 507508 / 506871 | 42/42 ok |
| 6 | both_prefix_relay_only | turing-20261114T044551Z-874d6a2b | 509423 / 508812 | 42/42 ok |
| 7 | both_prefix | turing-20261114T044706Z-34d34adb | 808023 / 706789 | 42/42 ok |
| 8 | both_iid | turing-20261114T044806Z-2a08b4ff | 962662 / 930985 | 42/42 ok |
| 9 | one_prefix | turing-20261114T044932Z-672fbb43 | 757730 / 755672 | 42/42 ok |
| 10 | one_iid | turing-20261114T045056Z-7e1658de | 706570 / 751397 | 42/42 ok |
| 11 | one_iid | turing-20261114T045253Z-23fe5c86 | 764602 / 742822 | 42/42 ok |
| 12 | both_iid | turing-20261114T045357Z-ae20c573 | 943815 / 933007 | 42/42 ok |
| 13 | both_prefix_relay_only | turing-20261114T045458Z-2538aeb0 | 508606 / 508002 | 42/42 ok |
| 14 | one_prefix | turing-20261114T045608Z-02477b72 | 668713 / 745319 | 42/42 ok |
| 15 | both_prefix | turing-20261114T045718Z-60f01e1e | 1516369 / 1705510 | 42/42 ok |

每场保留2个30秒setup与40个2秒caller真实读取、500ms节奏，各方向20个marker、原240秒case/300秒owner；raw attempt开始/结束、实际字节marker与original footer独立逐一核对。两端内核非loopback无IPv4，旧IPv6地址已不存在、新实际UDP源一致，IPv6 TCP到同一literal Relay前后可用；路由与CPU≥80% idle/PSI≤5/无并发Cargo、进程组/独立child所有权清理均核。

三场Relay-only在配对前安装owned namespace nft：仅UDP到独立IPv6 reflector39999放行，其他UDP丢弃；实际第二reflector39998先正后负控制、四份nft readback/计数和两侧实际Hop::Relay门保留。它证明已建立Relay在双端地址改变后恢复，**不是**从初始Direct突然阻断UDP的切换验收。

源与准备失败：产品CLI一直是已接受I4＋N2的SHA443c6e010293b83e7c0e963b6eb96edb3897dd4c0d6ada0bca9f6fca02666a43，产品/vendor变化0。前4场test源f3a62316/tree4ea140，完整netlab SHA72589f...；第5 v1在nft语法准备失败、业务0，原043417Z-fdd40789及analysis错误/拒绝记录全部保留，停止v1未继续。设计v2先冻后只修nft literal；新源4d6341a1/tree1cbe5c，producer043812Z-bfb5fd1c普通Release及实际source nft在独立user/netns安装＋读回均过，test SHA187f8487...；准确CLI字节不变。前后完整源除一条nft输入字符串外逐字相同，前4仅功能carryover；后11固定v2，不能当同ELF性能cohort。

原未格式准备、E0308编译失败和nft语法失败均留在来源链，未改期限、物理几何、原断言或替换坏业务样本。全部明细在E/ipv6-only-mobility-verification-v2.json、runtime-progress-v1/v2和functional-carryover-v2。

边界：IID采用add/DAD/delete，可能地址重叠，不是原子突发断旧地址。未覆盖NAT64/DNS64、冷新peer的发布发现、peer relay移动、睡眠/解锁、真实Wi-Fi↔蜂窝、移动端功耗/后台、Relay容量与故障域；不能把实验室秒数作为WAN或iPhone体验承诺。
