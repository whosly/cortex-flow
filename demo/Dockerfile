# ============================================================================
# CortexFlow Demo - Multi-stage Docker Build
# ============================================================================
# 从项目根目录构建: docker build -f demo/Dockerfile -t cortex-flow-demo .
# ============================================================================

# Stage 1: Build
FROM rust:1.77-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# 先复制整个项目（利用 Docker 层缓存需要更细粒度的 COPY，这里简化处理）
COPY . .

# 构建 demo 二进制
RUN cargo build --release -p cortex-flow-demo --bin cortex-flow-demo

# ============================================================================
# Stage 2: Runtime (minimal image)
# ============================================================================
FROM alpine:3.19

RUN apk add --no-cache ca-certificates curl && \
    addgroup -S app && adduser -S app -G app

WORKDIR /app

# 复制二进制
COPY --from=builder /app/target/release/cortex-flow-demo /app/cortex-flow-demo

# 复制前端静态文件
COPY demo/static /app/static

USER app

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
    CMD curl -sf http://localhost:3000/ > /dev/null || exit 1

ENTRYPOINT ["/app/cortex-flow-demo"]
