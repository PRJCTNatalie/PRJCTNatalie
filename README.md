# PRJCTNatalie

Foundation of **Natalie**, an AI agent named after her maintainer.

Built in Rust. Exposes both a CLI REPL and an HTTP API. Targets any
OpenAI-compatible backend.

## Build

```sh
cargo build            # debug
cargo build --release  # release
```

Heavy builds are expected to run on the local build server (see `AGENTS.md`).

## Configuration

All configuration comes from environment variables:

| Variable | Default | Description |
|---|---|---|
| `OPENAI_API_KEY` | (empty) | API key for the backend. |
| `OPENAI_BASE_URL` | `https://api.openai.com/v1` | Base URL of an OpenAI-compatible API. |
| `NATALIE_MODEL` | `gpt-4o-mini` | Model identifier. |
| `NATALIE_TEMPERATURE` | `0.7` | Sampling temperature. |
| `NATALIE_MAX_TOKENS` | `2048` | Max tokens per completion. |
| `NATALIE_SYSTEM_PROMPT` | (built-in) | Override the system prompt. |
| `RUST_LOG` | `info` | Log filter. |

## Usage

```sh
# Interactive REPL
natalie
# or
natalie chat

# HTTP server
natalie serve --host 127.0.0.1 --port 8080
# POST /chat  {"message": "..."}  -> {"reply": "..."}
# GET  /health
```

## Built-in tools

- `echo` — echo text (wiring test).
- `current_time` — UTC timestamp.
- `read_file` — read a UTF-8 file.
- `list_dir` — list directory entries.

## Layout

```
src/
  main.rs        # CLI entrypoint
  lib.rs         # crate root
  config/        # env-based configuration
  llm/           # OpenAI-compatible client + wire types
  tools/         # tool trait, registry, built-ins
  agent/         # agent loop with tool execution
  cli/           # REPL
  server/        # axum HTTP API
```

## Status

Foundation scaffold. Agent loop, tool execution, CLI, and HTTP API are wired
and build cleanly. No API key is bundled; set `OPENAI_API_KEY` to use.
