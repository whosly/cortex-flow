测试数据 Fixture 目录结构

tests/
├── _common/
│   └── fixture_loader.rs          # 共享 fixture 加载器 + 类型定义
├── fixtures/
│   ├── config/
│   │   ├── valid_config.json      # 有效配置 (JSON)
│   │   ├── valid_config.toml      # 有效配置 (TOML)
│   │   ├── invalid_zero_workers.json   # 无效: max_workers=0
│   │   ├── invalid_zero_timeout.json   # 无效: default_timeout=0
│   │   └── high_parallelism.json       # 边界: 高并行+debug+Sequential
│   ├── dag/
│   │   ├── linear_dag.json        # 线性 DAG (a→b→c)
│   │   ├── diamond_dag.json       # 菱形 DAG
│   │   ├── single_node_dag.json   # 单节点 DAG
│   │   ├── labeled_edge_dag.json  # 带边标签 DAG
│   │   ├── sequential_workflow.json  # 编排器线性工作流
│   │   └── parallel_workflow.json    # 编排器并行工作流
│   ├── llm/
│   │   ├── llm_configs.json       # OpenAI/Ollama/Azure 配置场景
│   │   ├── mock_responses.json    # 模拟 LLM 响应集合
│   │   └── token_scenarios.json   # Token 追踪场景
│   ├── task/
│   │   └── task_definitions.json  # 任务定义 (id/name/desc/tags)
│   ├── error_recovery/
│   │   └── retry_and_compensation.json  # 重试策略+补偿链
│   └── context/
│       └── scope_isolation.json   # 作用域隔离测试数据
