# Matrix Chat System
## Based on Conduit Matrix Server

This directory contains a Matrix chat system implementation built on [Conduit](https://conduit.rs/) - a lightweight, fast, and reliable Matrix chat server.

## Why Conduit?

- **Lightweight**: Single binary with embedded database
- **Fast**: Optimized performance compared to other Matrix servers  
- **Simple**: Easy setup and low system requirements
- **Open Source**: No commercial restrictions (unlike Element.io)
- **Rust-based**: Fits perfectly with our tech stack
- **RocksDB**: Embedded database, no external dependencies

## Architecture Integration

The chat system provides a complete Matrix server implementation:

### Authentication
- **OIDC Integration**: Users authenticate via OIDC-compatible identity providers
- **Single Sign-On**: Seamless login from web interfaces
- **Access Control**: Role-based access control

### Support Model
- **AI Chatbots**: Automated responses for common questions
- **Human Escalation**: Complex issues handled by support staff
- **Access Control**: Chat access based on user permissions

### Features
- **Community Chat**: Users share insights and experiences
- **Support Channels**: Direct access to customer support
- **Web Integration**: Chat accessible via web interfaces
- **Matrix Federation**: Can bridge to external chat systems
- **Audit Trails**: All conversations logged for compliance

## Implementation Status

**⚠️ This is a work-in-progress implementation (30% complete)**

The codebase has a solid architectural foundation with comprehensive type definitions, error handling, and testing. However, many core functions are currently stubs that need implementation.

**📋 See the detailed documentation:**
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) - What's implemented vs. missing
- [DEVELOPMENT_ROADMAP.md](DEVELOPMENT_ROADMAP.md) - Detailed development plan
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute and implement features

## Current Status

✅ **What Works:**
- Compiles successfully (Rust 1.75+)
- All 102 tests pass
- Complete type definitions and architecture
- OIDC integration framework
- Room management interfaces
- Federation client structure

🔄 **What Needs Implementation:**
- Authentication logic (JWT validation, user registration)
- Room operations (creation, joining, messaging)
- HTTP API endpoints
- Database persistence (currently in-memory)
- Matrix federation protocol

## Quick Start

```bash
# Build and test
cargo build
cargo test

# Check implementation status
cargo check  # Shows 16 warnings about unused code (normal for WIP)
```

## Resources

- **Conduit Website**: https://conduit.rs/
- **Documentation**: https://gitlab.com/famedly/conduit
- **Matrix Protocol**: https://matrix.org/
- **Community Chat**: #conduit:ahimsa.chat
