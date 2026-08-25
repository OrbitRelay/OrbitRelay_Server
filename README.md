# OrbitRelay Server

`OrbitRelay_Server` 是 OrbitRelay 服务端生态的 Monorepo，使用 Cargo Workspace 管理实时协作基础设施的核心模块。当前阶段不承载具体课堂、企业管理或其他业务系统。

## Workspace 结构

所有 crate 位于 `crates/`：

- `orbitrelay-core`：不依赖其他内部 crate 的基础类型层。
- `orbitrelay-protocol`：跨端协议模型定义。
- `orbitrelay-runtime`：协议 Action 的运行框架。
- `orbitrelay-sync`：Transport 无关的事件传播、过滤订阅和内存 EventBus。
- `orbitrelay-storage`：append-only EventStore、查询、回放游标和内存实现。
- `orbitrelay-node`：服务节点身份、状态、能力和 Registry 抽象。
- `orbitrelay-server`：配置、依赖组合、事件 Pipeline、生命周期和唯一二进制入口。

内部架构、协议、模块、开发流程和决策文档统一维护在根目录 `OrbitRelayDocs/`。