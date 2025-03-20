# Nginx日志QPS统计工具

[![Rust](https://img.shields.io/badge/Rust-1.84+-blue.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

用于分析Nginx访问日志并生成请求量时间序列图的Rust命令行工具。

## 功能特性

- 解析Nginx默认日志格式的时间戳
- 支持自定义统计时间间隔（秒级精度）
- 自动生成可视化折线图（PNG格式）
- 自动识别时间范围
- 支持中文字符显示

## 安装

1. 确保已安装[Rust工具链](https://www.rust-lang.org/tools/install)
2. 克隆本仓库：
```bash
git clone https://github.com/47sang/qps_rust.git
```
3. 构建项目：
```bash
cargo build --release
```

## 使用方法

```bash
cargo run --release -- \
    -l <日志文件路径> \
    -i <统计间隔(秒)> \
    -o <输出图片路径>
```

**参数说明**：
- `-l/--log-file`: Nginx访问日志文件路径（必需）
- `-i/--interval`: 统计时间间隔（秒，默认1秒）
- `-o/--output`: 输出图表文件名（默认qps_chart.png）

**示例**：
```bash
cargo run -- -l info-2025-03-18.log -i 300 -o 5min_qps.png
cargo run --release -- -l access.log -i 300 -o 5min_qps.png
```

## 日志格式要求
工具需要解析Nginx默认日志格式中的时间字段，日志格式示例：
```
127.0.0.1 - - [10/Jul/2023:14:23:45 +0800] "GET / HTTP/1.1" 200 612 "-" "curl/7.68.0"
```

## 技术栈
- 时间处理：[chrono](https://crates.io/crates/chrono)
- 命令行解析：[clap](https://crates.io/crates/clap)
- 数据可视化：[plotters](https://crates.io/crates/plotters)
- 正则解析：[regex](https://crates.io/crates/regex)

## 示例效果
![QPS Chart示例](demo300.png)
![QPS Chart示例](demo1.png)


# TODO:Nginx日志QPS统计工具潜在优化与功能扩展备忘录

## 当前问题与潜在优化点

### 性能优化
1. **大文件处理**：当前实现将所有时间戳加载到内存，对于GB级日志文件可能导致内存溢出
   - 考虑实现流式处理或分块读取
   - 可选择性地使用内存映射文件(memory-mapped files)技术

2. **并行处理**：
   - 使用Rayon库实现日志解析和统计的并行化
   - 对大文件实现多线程分段处理

### 功能扩展

1. **日志格式支持**：
   - 支持自定义日志格式配置
   - 增加对JSON格式日志的支持
   - 支持Apache、IIS等其他Web服务器日志格式

2. **数据分析增强**：
   - 添加HTTP状态码分布统计
   - 实现请求路径热点分析
   - 增加IP地址来源统计与地理位置可视化
   - 添加用户代理(User-Agent)分析

3. **可视化增强**：
   - 支持多种图表类型(柱状图、饼图等)
   - 添加交互式Web界面(可考虑使用wasm+yew)
   - 支持导出为SVG、PDF等多种格式
   - 实现多维度数据的仪表盘展示

4. **实时监控**：
   - 实现日志文件的实时监控(类似tail -f)
   - 添加阈值告警功能
   - 支持WebSocket推送实时数据到前端

### 用户体验改进

1. **配置管理**：
   - 支持配置文件(TOML/YAML)定义分析规则
   - 添加配置文件模板生成功能

2. **输出增强**：
   - 生成HTML报告
   - 支持将分析结果导出为CSV/Excel
   - 添加命令行进度条显示

3. **批处理能力**：
   - 支持批量处理多个日志文件
   - 实现日志文件的自动轮转检测与合并分析

## 技术债务与代码优化

1. **错误处理**：
   - 实现更细粒度的错误类型
   - 改进错误信息的可读性

2. **测试覆盖**：
   - 添加单元测试和集成测试
   - 创建基准测试评估性能

3. **代码结构**：
   - 将功能模块化，拆分为多个文件
   - 考虑使用特征(Trait)抽象不同的日志解析器

## 部署与分发

1. **打包**：
   - 提供预编译二进制文件
   - 创建Docker镜像便于在容器环境使用

2. **集成**：
   - 提供与Prometheus/Grafana集成的接口
   - 开发ELK Stack插件

## 长期规划

1. **机器学习集成**：
   - 实现异常检测算法
   - 添加流量预测功能

2. **分布式支持**：
   - 支持分布式日志收集与分析
   - 实现集群环境下的负载均衡
