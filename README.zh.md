# claude-sessions

通过 CLI 管理 Claude Code 会话 — 跨项目地列出、查看和删除会话。

## 安装

从 [Releases](https://github.com/javyxu/claude-sessions/releases) 下载对应平台的二进制，或从源码编译：

```bash
cargo build --release
# 二进制文件位于 target/release/claude-sessions
```

## 命令

| 命令 | 别名 | 说明 |
|------|------|------|
| `list` | `ls` | 列出当前项目的会话（默认 20 条） |
| `show <id>` | `info`、`inspect` | 查看会话详细信息 |
| `delete <id>` | `rm`、`remove` | 删除会话及关联文件 |
| `projects` | `prj` | 列出所有项目及会话数量 |
| `active` | `running` | 查看当前活跃会话 |

## 选项

| 选项 | 适用命令 | 说明 |
|------|----------|------|
| `--project <name>` | `list` | 按项目名模糊筛选 |
| `--limit N` | `list` | 限制返回 N 条结果 |
| `--json` | `list` | 以 JSON Lines 格式输出 |
| `--all` | `list` | 显示所有项目的会话 |
| `--force` | `delete` | 强制删除活跃会话 |

## 用法示例

```bash
# 查看当前项目会话
claude-sessions list

# 查看所有项目会话
claude-sessions list --all

# JSON 格式输出
claude-sessions list --json

# 搜索特定项目
claude-sessions list --project my-app

# 查看会话详情
claude-sessions show 24fc85db-xxxx-xxxx-xxxx-xxxxxxxxxxxx

# 删除会话
claude-sessions delete 24fc85db-xxxx-xxxx-xxxx-xxxxxxxxxxxx

# 强制删除活跃会话
claude-sessions delete 24fc85db-xxxx-xxxx-xxxx-xxxxxxxxxxxx --force

# 查看项目概览
claude-sessions projects

# 查看活跃会话
claude-sessions active
```

## 开发

```bash
# 编译 release 二进制
cargo build --release

# 二进制文件位于 target/release/claude-sessions
```

## 技术栈

- Rust
- serde / serde_json

## License

MIT
