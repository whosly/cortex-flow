# CortexFlow

基于 Rust 实现的AI 智能体调度框架，包括：智能体任务编排， 核心思路是将复杂目标拆解为一系列有序、可控、可评估的子任务，通过系统性的逻辑设计，确保每一步都清晰、稳定、可复用。
核心编排模式：顺序编排、MapReduce、DAG（有向无环图）、共识模式(让多个Agent独立处理同一任务，再通过投票等方式整合结果)、分层编排、制作者-检查者模式、基于Actor模型的并发编排


## 1. 项目概述

CortexFlow 是一个 Rust 实现的 AI 智能体调度框架。它把复杂目标拆成有序的子任务，按依赖关系编排执行。

### 编排模式

| 模式 | 说明 | 对应策略 |
|------|------|----------|
| 顺序编排 | 线性依次执行 | `SequentialStrategy` |
| DAG 编排 | 按有向无环图的依赖关系执行 | `DAGStrategy` |
| MapReduce | 分发到多个执行器后聚合结果 | 规划中 |
| 共识模式 | 多 Agent 独立处理同一任务，投票整合 | 规划中 |
| 分层编排 | 嵌套子 DAG 实现层级化编排 | 规划中 |
| 制作者-检查者 | 一个 Agent 生产、另一个验证 | 规划中 |

### 技术栈

| 领域 | 选型 |
|------|------|
| 异步运行时 | Tokio (`features = ["full"]`) |
| 序列化 | serde + serde_json + toml |
| 错误处理 | thiserror + anyhow |
| 异步 trait | async-trait |
| 并发原语 | parking_lot（同步锁） |
| 时间与 ID | chrono + uuid |
| 异步工具 | futures |
| LLM 客户端 | async-openai（可选） |
| 可观测性 | tracing + tracing-subscriber（可选） |


## 2. 编译与运行

### 2.1 编译



### 2.2 运行



### 2.3 代码质量

```

# 格式化
cargo fmt 

cargo clippy -- -D warnings

cargo clippy --tests -- -D warnings 

cargo test --lib --tests 

cargo test

```

### 2.4 使用例子

[demo](demo/README.md)


## 3. 项目结构

```
src/
├── lib.rs                    # 库入口，重新导出常用类型
├── main.rs                   # 二进制入口
├── dag/                      # DAG 编排核心
│   ├── mod.rs
│   ├── dag_impl.rs           # DAG 数据结构
│   ├── builder.rs            # DAGBuilder 流式构建器
│   ├── node.rs               # DAGNode 节点
│   ├── edge.rs               # DAGEdge 边
│   └── executor.rs           # DAGExecutor 并行执行器
├── task/                     # 任务抽象与执行
│   ├── mod.rs
│   ├── task_trait.rs         # Task / TaskExecutor trait
│   ├── task_input.rs         # TaskInput
│   ├── task_output.rs        # TaskOutput
│   ├── task_result.rs        # TaskResult
│   ├── task_metadata.rs      # TaskMetadata
│   ├── simple_task.rs        # SimpleTask 简易实现
│   └── registry.rs           # TaskRegistry 注册表
├── strategy/                 # 可扩展编排策略
│   ├── mod.rs                # StrategyType 枚举
│   ├── strategy_trait.rs     # OrchestrationStrategy trait
│   ├── dag_strategy.rs       # DAG 编排策略
│   ├── sequential_strategy.rs # 顺序编排策略
│   ├── factory.rs            # StrategyFactory 工厂
│   └── registry.rs           # StrategyRegistry 注册表
├── execution/                # 执行计划与引擎
│   ├── mod.rs
│   ├── engine.rs             # ExecutionEngine
│   ├── plan.rs               # ExecutionPlan / ExecutionNode
│   ├── state.rs              # ExecutionState / ExecutionStatus / ExecutionPhase
│   └── report.rs             # ExecutionReport / ReportGenerator
├── orchestrator/             # 框架门面
│   ├── mod.rs
│   ├── orchestrator.rs       # Orchestrator 编排器
│   └── builder.rs            # OrchestratorBuilder 构建器
├── context/                  # 执行上下文
│   ├── mod.rs
│   ├── context.rs            # ExecutionContext / ContextSnapshot / ContextLog
│   └── scope.rs              # ContextScope 作用域隔离
├── config/                   # 框架配置管理
│   ├── mod.rs
│   ├── framework_config.rs   # FrameworkConfig
│   └── manager.rs            # ConfigManager
├── llm/                      # LLM 客户端封装
│   ├── mod.rs
│   ├── client.rs             # LLMClient / LLMClientTrait / MockLLMClient
│   ├── config.rs             # LLMConfig / LLMProvider（含 custom() 快捷构造）
│   ├── message.rs            # Message / ChatRequest（请求级参数覆盖）
│   ├── registry.rs           # LLMClientRegistry（多模型注册表）
│   ├── response.rs           # LLMResponse / TokenUsage
│   └── token_tracker.rs      # TokenTracker（自动集成）
├── error_recovery/           # 错误处理与恢复
│   ├── mod.rs
│   ├── retry.rs              # RetryPolicy
│   ├── handler.rs            # ErrorHandler
│   ├── compensation.rs       # Compensation / CompensationChain
│   └── recovery.rs           # ErrorRecovery
├── observability/            # 可观测性
│   ├── mod.rs
│   ├── tracer.rs             # Tracer / TracerImpl / Span / TraceReport
│   └── metrics.rs            # MetricsCollector / Metric
└── error/                    # 统一错误类型
    └── mod.rs                # Error / Result
```

## 4. 架构设计

### 4.1 分层架构

```
┌──────────────────────────────────────────────┐
│            Orchestrator（编排层）              │  统一入口，DAG 构建、策略注册、LLM/可观测性集成
├──────────────────────────────────────────────┤
│            Strategy（策略层）                  │  DAG / Sequential 策略实现 + 工厂 + 注册表
├──────────────────────────────────────────────┤
│            Execution（执行层）                 │  ExecutionEngine + ExecutionPlan + 状态机 + 报告
├──────────────────────────────────────────────┤
│            DAG（图编排层）                     │  DAG / DAGBuilder / DAGExecutor / Node / Edge
├──────────────────────────────────────────────┤
│            Task（任务层）                      │  Task trait / TaskExecutor / TaskRegistry / SimpleTask
├──────────────────────────────────────────────┤
│            基础设施层                          │
│  ┌────────┐ ┌────────┐ ┌───────────┐        │
│  │ Error  │ │Context │ │  Config   │        │
│  └────────┘ └────────┘ └───────────┘        │
│  ┌────────┐ ┌────────┐ ┌───────────┐        │
│  │  LLM   │ │Observab│ │  Recovery │        │
│  └────────┘ └────────┘ └───────────┘        │
└──────────────────────────────────────────────┘
```

### 4.2 模块依赖

```
orchestrator ──→ strategy, dag, context, task
    │               │           │
    │               │           └──→ error
    │               └──→ execution ──→ dag, context, error
    │                        │
    │                        └──→ strategy
    │
    ├──→ config ──→ error
    ├──→ llm ──→ error
    ├──→ observability
    └──→ error_recovery ──→ context, error

dag ──→ context, task, error
task ──→ error
context ──→ error
```

### 4.3 模块职责

#### dag — DAG 编排核心
定义有向无环图结构、构建器与执行器，负责任务依赖关系建模与并行调度。
- `DAG` / `DAGBuilder` — DAG 数据结构与流式构建器，支持添加节点、边及拓扑验证
- `DAGNode` / `DAGEdge` — 节点（携带任务 ID 与标签）与边（支持标签标记数据流语义）
- `DAGExecutor` — 按拓扑层并行执行节点，自动收集上游输出传递给下游，支持进度回调与取消
- `NodeExecutionResult` — 节点执行结果（成功/失败、输出、耗时、时间戳）

#### task — 任务抽象与执行
定义 Task trait、任务注册表、输入输出与元数据，是框架中最小可调度单元。
- `Task`(trait) — 任务核心接口，定义 execute/validate/description 等方法
- `TaskExecutor` — 任务执行器，协调验证→执行→后处理流程
- `TaskRegistry` — 任务注册表，支持按 ID/名称查找与动态注册
- `TaskInput` / `TaskOutput` / `TaskResult` — 任务的输入数据、输出数据与执行结果
- `TaskMetadata` — 任务元数据（ID、名称、描述、标签）
- `SimpleTask` — 基于 `Box<dyn Fn>` 的简易任务实现

#### strategy — 可扩展编排策略
定义编排策略接口及 DAG/Sequential 等内置实现，支持通过工厂和注册表扩展。
- `OrchestrationStrategy`(trait) — 策略核心接口，定义 execute/validate/plan/rollback 方法
- `DAGStrategy` — DAG 编排策略，按拓扑排序分层并行执行
- `SequentialStrategy` — 顺序执行策略，按拓扑排序逐个执行（并发数为 1）
- `StrategyFactory` — 策略工厂，按 `StrategyType` 创建策略实例
- `StrategyRegistry` — 策略注册表，支持动态注册与按名称/类型查找
- `StrategyType` — 策略类型枚举（Dag/Sequential/MapReduce/Consensus/Hierarchical/ProducerChecker）

#### execution — 执行计划与引擎
管理执行状态、调度节点、生成执行报告。
- `ExecutionEngine` — 执行引擎，根据策略和 DAG 生成执行计划并驱动执行
- `ExecutionPlan` / `ExecutionNode` — 执行计划与节点，描述执行顺序与并行层级
- `ExecutionState` / `ExecutionStatus` / `ExecutionPhase` — 执行状态机（Pending→Running→Completed/Failed）
- `ExecutionReport` / `TaskReport` — 执行报告，汇总各节点结果与耗时
- `ReportGenerator` — 从执行结果自动生成报告，支持 Display 格式化输出

#### orchestrator — 框架门面
协调 DAG 构建、策略选择、执行引擎等组件，提供统一的高层 API。
- `Orchestrator` — 编排器主体，整合 DAG→Strategy→ExecutionEngine 的完整流程，支持多模型注册与 Token 聚合
- `OrchestratorBuilder` — 构建器，支持链式配置框架参数、策略、多模型注册（`with_llm`）、任务注册表等

#### context — 执行上下文
提供作用域隔离的键值存储、上下文快照与日志，支持父子作用域数据继承与覆盖，集成 LLM 客户端访问。
- `ExecutionContext` — 执行上下文，提供 get/set/remove 操作，支持作用域层级；集成 `get_llm()`/`default_llm()` 按名称获取 LLM 客户端
- `ContextScope` — 作用域隔离，子作用域可覆盖父作用域数据但不影响父级
- `ContextSnapshot` — 上下文快照，保存某一时刻的完整状态，支持回滚
- `ContextLog` — 上下文变更日志，记录 set/remove 操作历史

#### config — 框架配置管理
支持 JSON/TOML 格式的配置加载、验证与运行时管理。
- `FrameworkConfig` — 框架配置结构体（max_workers、default_timeout、策略类型、日志级别等）
- `ConfigManager` — 配置管理器，支持从文件/字符串加载配置，提供验证与默认值填充

#### llm — LLM 客户端封装
大语言模型客户端封装，支持多提供商、多模型管理、请求级参数覆盖与自动 Token 追踪。
- `LLMClient` / `LLMClientTrait` — LLM 客户端及 trait，支持聊天补全、流式输出与请求级参数覆盖
- `LLMConfig` / `LLMProvider` — 配置（API Key、模型、Base URL）与提供商枚举（OpenAI/Anthropic/Azure/Ollama/Custom）
- `LLMConfig::custom()` — 快捷构造器，用于任何 OpenAI 兼容 API（火山引擎豆包、DeepSeek 等）
- `ChatRequest` — 请求级别参数覆盖（model、temperature、max_tokens），无需创建新客户端
- `LLMClientRegistry` — 多模型注册表，按名称注册、查找、设置默认，聚合 Token 追踪
- `Message` — 消息类型（System/User/Assistant），支持内容与工具调用
- `LLMResponse` / `TokenUsage` — 响应结构与 Token 使用量统计
- `TokenTracker` — Token 消耗自动追踪器，累计 prompt/completion/total token 与成本估算（LLMClient 自动集成）
- `MockLLMClient` — 测试用 Mock 客户端，可预设响应与延迟
- `chat_structured` — 结构化输出函数，将 LLM 响应反序列化为指定类型

#### error_recovery — 错误处理与恢复
提供任务执行过程中的错误处理和恢复机制，整合重试、错误分类与补偿操作。
- `RetryPolicy` — 重试策略，支持固定间隔、指数退避，可配置抖动和可重试错误码
- `ErrorHandler` — 错误处理器，自动分类可恢复/不可恢复错误，支持错误码到动作的映射
- `Compensation`(trait) — 补偿操作 trait，支持同步/异步函数式补偿和补偿链
- `CompensationChain` — 补偿链，按注册逆序依次执行补偿操作（Saga 模式）
- `ErrorRecovery` — 统一管理器，整合重试+错误处理+补偿，提供 `execute_with_retry` 方法

#### observability — 可观测性
提供分布式追踪与指标收集能力。
- `Tracer`(trait) / `TracerImpl` — 追踪接口与实现，支持 Span 创建与嵌套
- `Span` / `SpanStatus` / `SpanEvent` — 追踪单元，记录开始/结束时间、状态(Ok/Error)与事件
- `TraceReport` — 追踪报告，汇总一次执行的所有 Span 数据
- `MetricsCollector` / `Metric` — 指标收集器，支持 Counter/Gauge/Histogram 三种类型

#### error — 统一错误类型
定义框架统一的错误类型和处理机制，涵盖 18 种错误场景。
- `Error`(enum) — 统一错误枚举（TaskExecution/TaskNotFound/DAGValidation/DAGCycle/LLMCall/Config/Context/Strategy/Serialization/Timeout/Cancelled/Resource/Validation/Internal/IO/JSON/TOML/Other）
- `Result<T>` — 框架结果类型别名
- 提供 `is_recoverable()` 可恢复性判断与 `error_code()` 错误码体系

### 4.4 数据流

```
用户请求
  │
  ▼
Orchestrator::execute_dag()
  ├── 1. DAGBuilder 构建 DAG
  ├── 2. dag.validate() 校验合法性
  ├── 3. ExecutionContext 创建执行上下文
  ├── 4. DAGExecutor 并行执行节点
  │     ├── Kahn 算法：入度为 0 的就绪节点入队
  │     ├── 按层并行，受 max_parallelism 限制
  │     ├── 上游输出 → 下游输入（单上游直传，多上游合并为 Object）
  │     ├── 进度回调 + 取消检查（AtomicBool）
  │     └── NodeExecutionResult 收集
  └── 5. 汇总结果 → ExecutionResult
```


## 5. 功能迭代的核心设计改动

### 5.1 202511 核心设计改动

| 决策 | 理由             | 
| ---- | -------------------------- |
| 策略运行时注册机制 | strategy/registry.rs， 新增 StrategyRegistry，支持 register()/create()/unregister()/is_registered()，默认注册 DAG 和 Sequential 两种策略 | 
| LLM 流式输出 | llm/client.rs， LLMClientTrait 新增 stream_chat() 方法，返回 Receiver<Result<StreamChunk>>，默认回退到普通 chat | 
| LLM 重试/超时 |  LLMClient.chat_with_config() 内置了超时（tokio::time::timeout）和指数退避重试，只有可恢复错误才重试 | 
| 上下文快照/恢复 | context/context.rs, xecutionContext 新增 snapshot()/restore()/clear()，ContextSnapshot 支持序列化  | 


### 5.2 202601 核心设计改动

| 决策 | 理由             | 
| ---- | -------------------------- |
| 注册机制 | strategy/registry.rs | 
| 验证完整性 | strategy/factory.rs | 
| 任务间数据传递 | DAGExecutor 的 node_outputs + ExecutionProgress + 进度回调 | 
| 结构化输出 | llm/client.rs | 
| Token追踪 | llm/token_tracker.rs | 
| 宏简化 | TaskRegistry（task/registry.rs），支持验证、元数据、标签/名称查找 | 
| 作用域隔离 | 增强 ContextScope 父子隔离（context/scope.rs, context/context.rs），ExecutionContext::snapshot()/restore() | 
| tracing集成 | TracerImpl, MetricsCollector, ReportGenerator, ExecutionProgress | 
| 环境变量覆盖、热更新  | ConfigManager::load_with_env_override(), reload(), 改进错误信息（config/manager.rs, config/framework_config.rs） | 
| src/task/registry.rs | 	TaskRegistry 独立实现 | 
| src/llm/token_tracker.rs	 | Token 消耗追踪 | 
| tests/task/	 | 任务模块集成测试  | 
| tests/error_recovery/ | 	错误恢复集成测试  | 
| tests/context/	 | 上下文集成测试  | 
| tests/config/	 | 配置集成测试  | 
| tests/llm/	 | LLM集成测试 | 


### 5.3 202603 核心设计改动

| 决策 | 理由             | 
| ---- | -------------------------- |
| tests/_common/ 目录   | Cargo 不会自动发现子目录中的文件，避免被编译为独立 test crate             | 
| #[path] 属性引入   | 每个测试模块通过 #[path = "../../_common/fixture_loader.rs"] 共享代码，无需复制            | 
| #![allow(dead_code)]  |   fixture_loader 中部分类型只在特定测试 crate 中使用，全局允许避免警告        | 
| JSON + TOML 双格式  |      config 模块天然需要测试两种解析格式     | 
| 类型化的 fixture  |   DAG/LLM/Task 有对应的 Rust struct，编译时检查 JSON schema 匹配       | 
| serde_json::Value 灵活解析  |     context 等简单场景直接用 Value 解析，避免过度类型化      |

### 5.4 202605 多模型架构设计

| 决策 | 理由             |
| ---- | -------------------------- |
| LLMConfig::custom() | 为 OpenAI 兼容 API（豆包、DeepSeek 等）提供快捷构造，替代手动构建 struct |
| ChatRequest 激活 | 已定义的 ChatRequest 集成到 LLMClientTrait，支持请求级别覆盖 model/temperature/max_tokens |
| LLMClientRegistry | HashMap 注册表替代单 `Option<Arc<dyn LLMClientTrait>>`，支持按名称注册/查找/默认设置 |
| Orchestrator 多模型集成 | Builder 新增 `with_llm()`/`with_default_llm()` 链式注册，保持 `with_llm_config()` 向后兼容 |
| TokenTracker 自动集成 | LLMClient.chat_with_config 成功后自动 record，LLMClientRegistry 共享 tracker 聚合统计 |
| ExecutionContext LLM 访问 | DAG 任务通过 `ctx.get_llm("name")`/`ctx.default_llm()` 获取客户端，无需 Arc 克隆 |
| build_override_config 独立函数 | 从 trait 方法提取为独立函数，保持 LLMClientTrait 的 dyn 兼容性 |


