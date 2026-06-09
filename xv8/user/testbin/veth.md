# Veth — Virtual Ethernet 測試

`veth` 測試驗證 xv8 的 veth（Virtual Ethernet）配對裝置。Veth 設備總是以配對形式出現，兩個端點像一條網路電纜的兩端——送入一端封包從另一端出現。這在容器網路中至關重要：容器內部有一端，宿主端連結到 bridge（如 docker0），讓容器與外部通訊。

## 相關文件

- [veth.md](../../kernel/src/net/veth.md) — 核心 veth 實作
- [interface.md](../../kernel/src/net/interface.md) — 網路介面抽象
- [container.md](./container.md) — 容器測試
