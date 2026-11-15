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
# GET  /hardware
# POST /deploy-local-ai  {}  -> deploys a local llama.cpp model

# Detect hardware + recommended model
natalie hardware

# Deploy a local AI (downloads a GGUF from HuggingFace, runs llama-server)
natalie deploy-local-ai
natalie deploy-local-ai --model "Llama 3.1 8B (Q4_K_M)"
```

## Built-in tools

- `echo` — echo text (wiring test).
- `current_time` — UTC timestamp.
- `read_file` — read a UTF-8 file.
- `list_dir` — list directory entries.

## Local AI ("Deploy local AI")

End-users (and spawned sub-agents) have two choices:

1. **Bring your own** — set `OPENAI_BASE_URL`/`OPENAI_API_KEY`/`NATALIE_MODEL`
   to any OpenAI-compatible backend.
2. **Deploy local AI** — run `natalie deploy-local-ai` (or `POST /deploy-local-ai`).
   Natalie detects the hardware (CPU cores, RAM, GPU/VRAM), picks the largest
   GGUF from a curated HuggingFace catalog that fits, downloads it, and launches
   `llama-server`. The endpoint is printed; point Natalie at it with
   `OPENAI_BASE_URL=<endpoint>/v1`.

Requires `llama-server` (llama.cpp) on `PATH`. If it is missing the command
fails with install guidance rather than silently breaking.

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
