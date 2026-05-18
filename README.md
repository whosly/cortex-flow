# CortexFlow

基于 Rust 实现的 AI 智能体调度框架

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
| 序列化 | serde + serde_json |
| 错误处理 | thiserror |
| DAG 引擎 | dagrs 0.8 |
| 并发原语 | parking_lot（同步锁）、dashmap（并发 Map） |
| LLM 客户端 | async-openai（可选） |
| 可观测性 | tracing（可选） |

---

## 2. 架构设计

### 2.1 分层架构

```
┌──────────────────────────────────────┐
│          Orchestrator（编排层）        │  统一入口，策略选择
├──────────────────────────────────────┤
│          Strategy（策略层）            │  Sequential / DAG
├──────────────────────────────────────┤
│          Execution（执行层）           │  ExecutionEngine + ExecutionPlan
├──────────────────────────────────────┤
│          DAG（图编排层）               │  DAG / DAGBuilder / DAGExecutor
├──────────────────────────────────────┤
│          Task（任务层）                │  TaskExecutor trait / SimpleTask
├──────────────────────────────────────┤
│          基础设施层                    │
│  ┌────────┐ ┌────────┐ ┌───────────┐ │
│  │ Error  │ │ Context│ │  Config   │ │
│  └────────┘ └────────┘ └───────────┘ │
│  ┌────────┐ ┌────────┐ ┌───────────┐ │
│  │  LLM   │ │Observab│ │  Recovery │ │
│  └────────┘ └────────┘ └───────────┘ │
└──────────────────────────────────────┘
```

### 2.2 模块依赖

```
orchestrator ──→ strategy ──→ execution ──→ dag ──→ task
    │               │             │          │
    │               │             │          └──→ error
    │               │             └──→ context
    │               └──→ config
    └──→ llm, observability, error_recovery
```

### 2.3 数据流

```
用户请求
  │
  ▼
Orchestrator::execute_dag()
  ├── 1. DAGBuilder 构建 DAG
  ├── 2. Strategy 选择执行策略
  ├── 3. ExecutionEngine 生成执行计划
  ├── 4. DAGExecutor 并行执行节点
  │     ├── 调度就绪节点（入度为 0）
  │     ├── 上游输出作为下游输入
  │     └── 进度回调 + 取消检查
  └── 5. 汇总结果 → ExecutionResult
```
