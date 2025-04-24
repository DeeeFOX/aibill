# Coze Token Service

This Rust service provides a simple endpoint (`/token`) to:
1.  Generate a JWT token signed with an RSA private key based on input parameters.
2.  Immediately exchange this JWT with the Coze API (`https://api.coze.cn/api/permission/oauth2/token`) for a final access token.

It's designed for secure handling of credentials (loading private key and API keys from environment variables) and easy deployment via Docker.

## Features

*   Generates RS256 signed JWT tokens compatible with Coze API requirements.
*   Exchanges the generated JWT for a Coze access token.
*   Configurable token duration.
*   Loads private key (`JWT_PRIVATE_KEY`), expected API key (`EXPECTED_COZE_API_KEY`), and Coze API URL (`COZE_API_URL`) securely from environment variables.
*   Supports `.env` file for local development configuration.
*   Dockerized for easy deployment.

## Prerequisites

*   Rust (latest stable version recommended, built with 1.86)
*   Docker & Docker Compose (for containerized deployment/testing)

## Setup & Running Locally (Without Docker)

1.  **Navigate to the service directory:**
    ```bash
    cd coze_token_service
    ```

2.  **Create `.env` file:**
    Create a file named `.env` inside this directory (`coze_token_service/.env`) with your RSA private key and API keys:
    ```dotenv
    # coze_token_service/.env
    JWT_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----
    YOUR_RSA_PRIVATE_KEY_CONTENT_HERE_INCLUDING_NEWLINES
    -----END PRIVATE KEY-----"

    EXPECTED_COZE_API_KEY=your_coze_api_key_here
    COZE_API_URL=https://api.coze.cn/api/permission/oauth2/token
    ```
    *Replace placeholders with your actual values.*
    *(Consider creating an `.env.example` file based on this structure.)*

3.  **Build the project:**
    ```bash
    cargo build
    ```

4.  **Run the service:**
    ```bash
    cargo run
    ```
    The service will start and listen on `0.0.0.0:9000`.

## Running Locally (With Docker Compose)

This is the recommended way for local testing.

1.  **Ensure `.env` file exists:** Make sure you have the `coze_token_service/.env` file created as described above.

2.  **Navigate to the project root** (the directory containing `docker-compose.yml`).

3.  **Build and run:**
    ```bash
    docker compose up --build -d
    ```
    This uses the `docker-compose.yml` in the root directory, which is configured to:
    *   Build the Docker image using `coze_token_service/Dockerfile`.
    *   Start a container named `coze-token-service-dev`.
    *   Map port 9000 on your host to port 9000 in the container.
    *   Load variables from `coze_token_service/.env` into the container's environment.

4.  **Check logs (optional):**
    ```bash
    docker compose logs -f token-service
    ```

5.  **Stop the service:**
    ```bash
    docker compose down
    ```

## API Endpoint

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

    Use your cloud provider's secret management tools.

## Project Structure (`coze_token_service/`)

```
coze_token_service/
├── .dockerignore     # Files to ignore during Docker build
├── .env              # Local environment variables (ignored by git)
├── .gitignore        # Files ignored by git
├── Cargo.lock
├── Cargo.toml        # Rust project manifest
├── Dockerfile        # Defines the Docker image build process
├── README.md         # This file
└── src/
    └── main.rs       # Main application source code
└── tests/            # Integration/unit tests (TODO)
