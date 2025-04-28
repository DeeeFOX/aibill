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
│   │   ├── auth/
│   │   ├── config/
│   │   ├── database/
│   │   ├── error/
│   │   ├── format/
│   │   ├── models/
│   │   ├── routes/
│   │   ├── services/
│   │   ├── tests/
│   │   └── main.rs
│   └── tests/          # Integration/unit tests
│       └── integration_tests.rs
├── coze_agent_service/ # Python utilities for Coze
│   ├── .env.example    # Example environment file
│   ├── README.md       # Utility documentation
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

## Running Locally (With Docker Compose)

This is the recommended way for local development and testing.

1.  **Ensure `.env` file exists:** Create a file named `.env` inside the `coze_token_service/` directory (`coze_token_service/.env`) with the necessary environment variables for the service. An example file `.env.example` is provided as a reference.

2.  **Navigate to the project root** (the directory containing `docker-compose.yml`).

3.  **Build and run the service:**
    ```bash
    docker compose up --build -d
    ```
    This command will:
    *   Build the Docker image for the `token-service` using the `Dockerfile` in `./coze_token_service`.
    *   Tag the image as `coze-token-service:local`.
    *   Create and start a container named `coze-token-service-dev`.
    *   Map port 9000 on your host to port 9000 in the container.
    *   Load environment variables from `./coze_token_service/.env` into the container.

4.  **Check logs (optional):**
    ```bash
    docker compose logs -f token-service
    ```

5.  **Stop the service:**
    ```bash
    docker compose down
    ```

## API Endpoints

### POST /token

Generates a JWT token and immediately exchanges it with the Coze API for a final access token.

*   **Method:** `POST`
*   **URL:** `http://localhost:9000/token` (adjust host/port if deployed elsewhere)
*   **Headers:**
    *   `Content-Type: application/json`
*   **Request Body (JSON):**
    ```json
    {
      "public_key": "your_public_key_identifier",
      "coze_api_key": "the_api_key_to_send",
      "duration_seconds": <integer, optional>
    }
    ```
    *   `public_key`: (Required) An identifier for the public key corresponding to the private key used for signing. This will be included in the JWT header's `kid` field.
    *   `coze_api_key`: (Required) The API key to authenticate the request. Must match the `EXPECTED_COZE_API_KEY` environment variable set in the service.
    *   `duration_seconds`: (Optional) The desired validity duration of the token in seconds. Defaults to 86400 (24 hours) if omitted.
*   **Success Response (200 OK):**
    *(This is the response directly from the Coze API)*
    ```json
    {
      "access_token": "czs_...", // The final access token from Coze
      "expires_in": 1745569363, // Expiry time provided by Coze
      "token_type": "Bearer"
    }
    ```
*   **Error Responses:**
    *   `400 Bad Request`: Invalid JSON or missing required fields in the request to *this* service.
    *   `401 Unauthorized`: Provided `coze_api_key` does not match `EXPECTED_COZE_API_KEY`.
    *   `500 Internal Server Error`: Issue generating the initial JWT or unexpected failure calling Coze API.
    *   `502 Bad Gateway`: Coze API returned an error during token exchange. Body contains Coze error.

*   **Example (cURL):**
    ```bash
    # Replace placeholders with actual values
    PUBLIC_KEY_ID="your_key_id"
    COZE_API_KEY_TO_SEND="your_coze_api_key_here" # Use the key configured in .env

    curl -X POST http://localhost:9000/token \
    -H "Content-Type: application/json" \
    -d '{
      "public_key": "'"$PUBLIC_KEY_ID"'",
      "coze_api_key": "'"$COZE_API_KEY_TO_SEND"'",
      "duration_seconds": 3600
    }'
    ```

### POST /resend

Resends a previously failed request to a specified URL.

*   **Method:** `POST`
*   **URL:** `http://localhost:9000/resend` (adjust host/port if deployed elsewhere)
*   **Headers:**
    *   `Content-Type: application/json`
*   **Request Body (JSON):**
    ```json
    {
      "location": "https://open.feishu.cn/open-apis/bitable/v1/apps/DC1sb1XABavEDfszxSpcyEtankg/tables/tbl5kDOiahgZrtCO/records/batch_create",
      "headers": {
          "Authorization": "Bearer xxxx"
      },
      "params": {
          "records": [
              {
                  "fields": "{\"收支类型\":\"支出\",\"款项金额\":14.91,\"消费类型\":\"交通\",\"流水说明\":\"财付通-滴滴出行\",\"日期时间\":1745809165,\"收入方\":\"滴滴出行\",\"支出方\":\"用户\",\"备注\":\"\"}"
              }
          ]
      },
      "commands": {
          "json_parse": ["$.params.records[*].fields"]
      }
    }
    ```
    *   `location`: (Required) The URL to resend the request to.
    *   `headers`: (Optional) A JSON object of headers to include in the resend request.
    *   `params`: (Optional) A JSON object of parameters to include in the resend request.
    *   `commands`: (Optional) A JSON object of commands to process the response.

*   **Success Response (200 OK):**
    *(This is the response from the `location` URL)*
    ```json
    {
      // Response body from the resend location
    }
    ```
*   **Error Responses:**
    *   `400 Bad Request`: Invalid JSON or missing required fields in the request to *this* service.
    *   `500 Internal Server Error`: Issue making the resend request or processing commands.
    *   `502 Bad Gateway`: The `location` URL returned an error. Body contains the error from the `location` URL.

*   **Example (cURL):**
    ```bash
    curl -i -X POST 'http://localhost:9000/resend' \
        -H 'Content-Type: application/json' \
        -d '{
        "location": "https://open.feishu.cn/open-apis/bitable/v1/apps/DC1sb1XABavEDfszxSpcyEtankg/tables/tbl5kDOiahgZrtCO/records/batch_create",
        "headers": {
            "Authorization": "Bearer xxxx"
        },
        "params": {
            "records": [
                {
                    "fields": "{\"收支类型\":\"支出\",\"款项金额\":14.91,\"消费类型\":\"交通\",\"流水说明\":\"财付通-滴滴出行\",\"日期时间\":1745809165,\"收入方\":\"滴滴出行\",\"支出方\":\"用户\",\"备注\":\"\"}"
                }
            ]
        },
        "commands": {
            "json_parse": ["$.params.records[*].fields"]
        }
    }
    ```

## Deployment (Production/Testing)

1.  **Build the Docker Image:**
    From the repository root:
    ```bash
    docker build -t your-registry/coze-token-service:latest ./coze_token_service
    ```
    *(Replace `your-registry/coze-token-service:latest` with your image tag)*

2.  **Push the Image:**
    Push to your container registry (e.g., Aliyun ACR).
    ```bash
    docker push your-registry/coze-token-service:latest
    ```

3.  **Run the Container:**
    When running the container (e.g., Kubernetes, ECS, etc.), ensure the following environment variables are securely injected:
    *   `JWT_PRIVATE_KEY`: The content of your private key PEM file.
    *   `EXPECTED_COZE_API_KEY`: The API key the service should expect.
    *   `COZE_API_URL`: The Coze token exchange endpoint URL.
    *   `RUST_LOG`: Set this to control logging levels (e.g., `info`, `debug`, `trace`).

    Use your cloud provider's secret management tools.

## License

(TODO: Add a LICENSE file - e.g., MIT, Apache-2.0)

## Contributing

(TODO: Add CONTRIBUTING.md if desired)
