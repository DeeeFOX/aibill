# AIBill Project Hub

This repository contains services and utilities related to the AIBill project.

## Components

1.  **Coze Token Service (`coze_token_service/`)**:
    *   A Rust-based web service that generates JWT tokens signed with an RSA private key and exchanges them for access tokens via the Coze API.
    *   Designed for secure handling of credentials and deployment via Docker.
    *   See the [Coze Token Service README](./coze_token_service/README.md) for detailed setup, API documentation, and deployment instructions.

2.  **Coze Agent Service Utilities (`coze_agent_service/`)**:
    *   Contains utility scripts, currently including a Python script (`getjwttoken_util.py`) for generating JWTs compatible with the Coze API for testing or specific use cases.
    *   See the [Coze Agent Service README](./coze_agent_service/README.md) for details on the utilities.

## Repository Structure

```
.
├── .gitignore          # Root gitignore rules
├── coze_token_service/ # Rust JWT generation and exchange service
│   ├── .dockerignore
│   ├── .env.example    # Example environment file
│   ├── .gitignore      # Service-specific gitignore
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── README.md       # Detailed service documentation
│   ├── src/
│   │   └── main.rs
│   └── tests/          # Integration/unit tests
│       └── integration_tests.rs
├── coze_agent_service/ # Python utilities for Coze
│   ├── .env.example    # Example environment file (TODO: Create this)
│   ├── README.md       # Utility documentation (TODO: Create this)
│   ├── getjwttoken_util.py
│   └── res/            # Resource files (keys ignored by git)
│       ├── .private_key.pem # (Ignored by git)
│       ├── .public_key.pem  # (Ignored by git)
│       └── ...
├── docker-compose.yml  # For local development using Docker
└── README.md           # This file (repository overview)
```

## Getting Started

Refer to the README file within each component's directory for specific instructions. For local development involving the Rust service, ensure Docker and Docker Compose are installed and follow the steps in `coze_token_service/README.md`.

## License

(TODO: Add a LICENSE file - e.g., MIT, Apache-2.0)

## Contributing

(TODO: Add CONTRIBUTING.md if desired)
