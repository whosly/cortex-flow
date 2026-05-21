# CortexFlow 数据分析管道 Demo

这是一个使用 CortexFlow 框架构建的数据分析管道示例项目，展示框架的核心功能。

支持两种运行模式：**CLI 模式**（终端输出）和 **Web 模式**（浏览器 UI）。

## 运行方式

### CLI 模式

```bash
# 进入 demo 目录
cd demo

# 运行基础数据管道（无 LLM）
cargo run -- basic

# 运行带 LLM 分析的管道（使用 Mock）
cargo run -- llm

# 运行错误恢复与重试演示
cargo run -- error-recovery

# 运行执行上下文演示
cargo run -- context

# 运行带进度回调的管道
cargo run -- progress

# 运行多模型 LLM 配置与切换演示
cargo run -- multi-model

# 运行所有演示（不含 multi-model）
cargo run -- all

# 查看帮助
cargo run -- --help
```

### Web 模式

```bash
# 启动 Web UI（默认端口 3000）
cargo run

# 访问 
http://localhost:3000
```

Web UI 提供：
- **Dashboard** — 统计卡片 + 管道快速入口
- **Pipeline Runner** — 选择管道 → SSE 实时进度条 + 日志 + 执行结果
- **DAG Visualizer** — Mermaid.js 交互式 DAG 图
- **LLM Models** — 已注册模型列表
- **Token Tracker** — Token 使用统计

### Docker 部署

```bash
# 从项目根目录构建
docker build -f demo/Dockerfile -t cortex-flow-demo .

# 运行
docker run -p 3000:3000 cortex-flow-demo
```

## 演示内容

数据分析管道 demo，包含 6 个可独立运行的演示：

| 模式 | 命令 | 展示功能 |
|------|------|----------|
| basic | `cargo run -- basic` | 基础 DAG 编排（6节点，并行+依赖） |
| llm | `cargo run -- llm` | LLM 集成（MockLLMClient） |
| error-recovery | `cargo run -- error-recovery` | 重试策略、重试耗尽、DAG 节点失败 |
| context | `cargo run -- context` | 上下文读写、日志、快照、DAG 数据传递 |
| progress | `cargo run -- progress` | 进度回调、进度条可视化 |
| multi-model | `cargo run -- multi-model` | 多模型 LLM 配置（GPT-4/Llama3/豆包）与切换 |


### 1. 基础 DAG 数据管道 (`basic`)

展示 CortexFlow 最核心的 DAG 编排能力：
- 6 个任务节点按依赖关系执行
- `数据清洗` 和 `数据验证` 并行执行
- `报告生成` 和 `可视化导出` 并行执行
- 上游输出自动传递给下游

```
% cargo run -- basic
========================================================
  CortexFlow 数据分析管道 Demo
========================================================

--- Demo 1: 基础 DAG 数据管道 ---

DAG 结构:
  [数据获取] ──┬──> [数据清洗] ──┬──> [数据分析] ──┬──> [报告生成]
               └──> [数据验证] ──┘                  └──> [可视化导出]

  [fetch_data] 获取到 10 条销售记录
  [clean_data] 清洗完成: 10 条记录, 3 个区域
  [validate_data] 数据验证: 通过
  [analyze_data] 分析完成: 趋势=下降, 增长率=-54.9%
  [generate_report] 报告已生成: 数据分析报告 - 2026-05-20 (基础模式)
  [export_visual] 可视化图表已导出 (121 字符)

--- 执行结果 ---
整体状态: 成功
总耗时: 0ms
节点结果:
  [OK] fetch_data (获取销售数据) - 0ms
  [OK] clean_data (清洗数据) - 0ms
  [OK] validate_data (验证数据) - 0ms
  [OK] analyze_data (分析数据) - 0ms
  [OK] generate_report (生成报告) - 0ms
  [OK] export_visual (可视化导出) - 0ms
```

### 2. 带 LLM 分析的管道 (`llm`)

展示 LLM 集成能力：
- 使用 `MockLLMClient` 模拟 LLM 调用
- 演示如何通过 Orchestrator 配置 LLM
- 生产环境可替换为 `LLMConfig::openai()` 或 `LLMConfig::ollama()`

```
% cargo run -- llm
========================================================
  CortexFlow 数据分析管道 Demo
========================================================

--- Demo 2: 带 LLM 分析的数据管道 ---

  [fetch_data] 获取到 10 条销售记录
  [clean_data] 清洗完成: 10 条记录, 总额 19650.00
  [llm_analyze] 发送数据摘要到 LLM: 10 条记录, 总金额 19650.00
  [generate_report] 智能报告已生成: 数据分析报告 - 2026-05-20 (LLM 智能分析)

--- 执行结果 ---
整体状态: 成功
总耗时: 0ms
节点结果:
  [OK] fetch_data (获取销售数据) - 0ms
  [OK] clean_data (清洗数据) - 0ms
  [OK] llm_analyze (LLM 智能分析) - 0ms
  [OK] generate_report (生成智能报告) - 0ms
```

### 3. 错误恢复与重试 (`error-recovery`)

展示三种错误恢复场景：
- **指数退避重试**：首次失败，第二次成功
- **重试耗尽**：始终失败，达到最大重试次数
- **DAG 节点失败**：中间节点失败导致后续节点无法执行

```
% cargo run -- error-recovery
========================================================
  CortexFlow 数据分析管道 Demo
========================================================

--- Demo 3: 错误恢复与重试 ---

场景1: 指数退避重试（模拟第2次成功）
  第 1 次尝试: 失败
  第 2 次尝试: 成功!
  重试结果: Ok("数据获取成功")

场景2: 重试耗尽（始终失败）
  第 1 次尝试: 失败
  第 2 次尝试: 失败
  第 3 次尝试: 失败
  重试结果: Err(LLM call failed: API rate limit)

场景3: DAG 中某个节点失败
  [step1] 执行成功
  [step2] 执行失败!
  [step3] 执行成功
  DAG 执行结果: success=false
    节点 step1: 成功 - "无错误"
    节点 step2: 失败 - "Task execution failed: 数据格式错误"
    节点 step3: 成功 - "无错误"
```

### 4. 执行上下文 (`context`)

展示 ExecutionContext 的使用：
- 键值存储的读写操作
- 日志记录与查询
- 快照与恢复
- DAG 任务间的数据传递（依赖传递 vs 上下文传递）

```
% cargo run -- context
========================================================
  CortexFlow 数据分析管道 Demo
========================================================

--- Demo 4: 执行上下文与数据传递 ---

Session ID: 90c66cc2-f968-40ee-a41d-6599989e2af3
设置上下文数据:
  pipeline_name = 数据分析管道
  config = {"max_retries":3,"threshold":0.8}
  batch_size = 100

上下文日志:
  [info] 管道启动
  [debug] 数据获取完成 [task: fetch_data]
  [warn] 数据质量较低 [task: validate_data]

添加临时数据后: 4 个键
恢复快照后: 3 个键
临时数据是否存在: false

--- DAG 中的数据传递 ---
  [producer] 生成数据
  [consumer] 依赖传递: direct=通过依赖传递, value=42

--- 执行结果 ---
整体状态: 成功
总耗时: 0ms
节点结果:
  [OK] producer (数据生产者) - 0ms
  [OK] consumer (数据消费者) - 0ms
```

### 5. 进度回调 (`progress`)

展示实时进度监控：
- 进度条实时更新
- 成功/失败计数
- 并行任务的执行时序可视化

```
% cargo run -- progress
========================================================
  CortexFlow 数据分析管道 Demo
========================================================

--- Demo 5: 带进度回调的管道 ---

  [validate] 完成
  [fetch] 完成
  [clean] 完成
  进度: [███████████░░░░░░░░░] 50% (3/6 成功:3 失败:0)
  [analyze] 完成
  进度: [██████████████░░░░░░] 66% (4/6 成功:4 失败:0)
  [export] 完成
  [report] 完成
  进度: [████████████████████] 100% (6/6 成功:6 失败:0)

--- 执行结果 ---
整体状态: 成功
总耗时: 1157ms
节点结果:
  [OK] fetch (获取数据) - 201ms
  [OK] validate (验证数据) - 151ms
  [OK] clean (清洗数据) - 302ms
  [OK] analyze (分析数据) - 401ms
  [OK] export (导出结果) - 101ms
  [OK] report (生成报告) - 201ms
```

### 6. 多模型 LLM 配置 (`multi-model`)

CortexFlow 多模型全特性：

- **LLMConfig::custom()** - 为火山引擎豆包、DeepSeek 等 OpenAI 兼容 API 提供快捷构造
- **ChatRequest** - 请求级别覆盖 model/temperature/max_tokens，无需创建新客户端
- **LLMClientRegistry** - HashMap 注册表，按名称注册/查找/设置默认模型
- **Orchestrator 多模型集成** - `with_llm()`/`with_default_llm()` 链式注册
- **TokenTracker 自动集成** - 调用后自动记录 Token，注册表级聚合统计
- **ExecutionContext LLM 访问** - DAG 任务通过 `ctx.get_llm()` 获取客户端，零 Arc 克隆

```
% cargo run -- multi-model
========================================================
  CortexFlow 数据分析管道 Demo
========================================================

--- Demo 6: 多模型 LLM 配置与切换 ---

=== 1. LLMConfig::custom() 快捷构造 ===

[火山引擎豆包] 使用 LLMConfig::custom() 构造
  provider: Custom
  model: doubao-pro-32k
  base_url: Some("https://ark.cn-beijing.volces.com/api/v3")

[DeepSeek] 使用 LLMConfig::custom() 构造
  provider: Custom
  model: deepseek-chat

=== 2. ChatRequest 请求级参数覆盖 ===

默认配置 model: gpt-4
ChatRequest 覆盖: model=Some("gpt-3.5-turbo"), temperature=Some(0.3), max_tokens=Some(512)

=== 3. Orchestrator Builder 多模型注册 ===

已注册模型: ["llama3", "doubao", "gpt4"]
默认模型: Some("gpt4")
获取 gpt4: true
获取 doubao: true

=== 4. ExecutionContext LLM 访问（推荐方式）===

DAG 结构（任务通过 ctx.get_llm() 获取模型）:
  [数据获取] ──> [数据清洗] ──┬──> [GPT-4 深度分析]  ──┐
                             ├──> [Llama3 本地校验]  ──┼──> [豆包报告润色] ──> [Token 统计]
                             └──> [数据统计]         ──┘

  [fetch_data] 获取 10 条销售记录
  [clean_data] 清洗完成: 10 条记录, 总额 19650.00
  [gpt4_analysis] 使用 GPT-4 分析...
  [llama3_validate] 使用本地 Llama3 校验...
  [statistics] 计算统计指标...
  [doubao_report] 使用豆包润色报告...
  [token_summary] Token 使用统计:
    调用次数: 0
    总 Token: 0
    估算成本: $0.0000
  [token_summary] 可用模型: ["llama3", "doubao", "gpt4"]

=== 5. TokenTracker 聚合统计 ===

所有模型聚合 Token 使用:
=== Token Usage Summary ===
Calls: 0
Prompt Tokens: 0
Completion Tokens: 0
Total Tokens: 0
Estimated Cost: $0.0000

=== 6. LLMClientRegistry 独立使用 ===

注册表中的模型: ["llama3", "doubao", "gpt4"]
默认模型: Some("gpt4")
包含 gpt4: true
模型数量: 3

=== 7. CortexFlow 多模型特性总结 ===

P0: LLMConfig::custom()      - 豆包/DeepSeek 等 OpenAI 兼容 API 快捷构造
P1: ChatRequest               - 请求级别覆盖 model/temperature/max_tokens
P2: LLMClientRegistry         - HashMap 注册表，按名称管理多模型
P3: Orchestrator 多模型集成    - Builder.with_llm()/with_default_llm() 链式注册
P4: TokenTracker 自动集成     - 调用后自动记录，注册表级聚合统计
P5: ExecutionContext LLM 访问 - ctx.get_llm()/ctx.default_llm() 零 Arc 克隆

--- 推荐用法 ---
1. Orchestrator::builder().with_llm("name", config).build()
2. DAG 任务中: ctx.get_llm("name") 获取客户端
3. 临时切换: ChatRequest::new(msg).with_model("other-model")
4. 查看成本: orchestrator.token_snapshot()
```

#### 推荐用法

```rust
// 1. Builder 注册多模型
let orchestrator = Orchestrator::builder()
    .with_llm("gpt4", LLMConfig::openai("sk-xxx", "gpt-4"))
    .with_llm("llama3", LLMConfig::ollama("llama3"))
    .with_llm("doubao", LLMConfig::custom("key", "doubao-pro-32k", "https://ark.cn-beijing.volces.com/api/v3"))
    .with_default_llm("gpt4")
    .build()
    .await?;

// 2. DAG 任务中通过 ctx 获取模型
orchestrator.execute_dag(|dag| {
    dag.add_node(SimpleTask::new("analyze", "分析", |_input, ctx| {
        let client = ctx.get_llm("gpt4").unwrap();
        Box::pin(async move {
            let response = client.chat(messages).await?;
            Ok(serde_json::to_value(&response).unwrap_or_default())
        })
    }));
}).await?;

// 3. 查看聚合 Token 使用
println!("{}", orchestrator.token_snapshot());
```

## 项目结构

```
demo/
├── Cargo.toml          # 项目配置（依赖 cortex-flow）
├── Dockerfile          # Multi-stage Docker 构建
├── .dockerignore
├── README.md           # 本文件
├── static/
│   └── index.html      # Web UI（Tailwind CSS + Mermaid.js）
└── src/
    ├── main.rs         # 入口：CLI / Web 模式分发
    ├── cli.rs          # CLI 模式演示（6 个 Demo）
    ├── handlers.rs     # Web API 处理器（6 个端点）
    ├── pipeline.rs     # 管道定义 + 执行逻辑
    ├── models.rs       # 数据模型
    └── state.rs        # 共享状态
```

## API 端点

| 端点 | 方法 | 功能 |
|------|------|------|
| `/api/pipelines` | GET | 管道列表 |
| `/api/pipelines/run?pipeline=id` | GET | SSE 实时执行管道 |
| `/api/pipelines/result` | GET | 最近执行结果 |
| `/api/dag/:name` | GET | Mermaid DAG 定义 |
| `/api/llm/models` | GET | 已注册 LLM 模型 |
| `/api/tokens` | GET | Token 使用统计 |

## 关键代码说明

### 创建编排器

```rust
let orchestrator = Orchestrator::builder()
    .with_max_parallelism(4)
    .build()
    .await?;
```

### 定义任务

```rust
let task = SimpleTask::new("task_id", "任务名称", |input, ctx| {
    Box::pin(async move {
        // 处理逻辑
        Ok(serde_json::json!({ "result": "done" }))
    })
});
```

### 构建 DAG

```rust
orchestrator.execute_dag(|dag| {
    dag.add_node(task1);
    dag.add_node(task2);
    dag.add_dependency("task1", "task2");  // task1 先执行，输出传给 task2
}).await?;
```

## 扩展指南

### 接入真实 LLM

将 `MockLLMClient` 替换为真实配置：

```rust
use cortex_flow::llm::{LLMClient, LLMConfig, LLMClientRegistry};

// 方式一：通过 Orchestrator Builder（推荐）
let orchestrator = Orchestrator::builder()
    .with_llm("gpt4", LLMConfig::openai("sk-xxx", "gpt-4"))
    .with_llm("llama3", LLMConfig::ollama("llama3"))
    // 使用 LLMConfig::custom() 快捷接入 OpenAI 兼容 API
    .with_llm("doubao", LLMConfig::custom(
        "your-volcengine-api-key",
        "doubao-pro-32k",
        "https://ark.cn-beijing.volces.com/api/v3",
    ))
    .with_default_llm("gpt4")
    .build()
    .await?;

// 方式二：独立使用 LLMClientRegistry
let registry = LLMClientRegistry::new();
registry.register_with_config("gpt4", LLMConfig::openai("sk-xxx", "gpt-4"));
registry.register_with_config("doubao", LLMConfig::custom("key", "doubao-pro-32k", "https://ark.cn-beijing.volces.com/api/v3"));

// 在 Cargo.toml 中启用 llm feature:
// cortex-flow = { path = "..", features = ["llm"] }
```

### 添加自定义任务

实现 `TaskExecutor` trait：

```rust
struct MyTask;
#[async_trait]
impl TaskExecutor for MyTask {
    async fn execute_task(&self, input: Value, ctx: &ExecutionContext) -> Result<Value> {
        Ok(json!({ "custom": true }))
    }
    fn task_id(&self) -> &str { "my_task" }
    fn task_name(&self) -> &str { "我的任务" }
}
```

### 添加错误恢复

```rust
let recovery = ErrorRecovery::new(
    RetryPolicy::exponential(3, 1000, 2.0)
);
let result = recovery.execute_with_retry(|| async {
    some_unreliable_operation().await
}).await?;
```
