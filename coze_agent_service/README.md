# Coze Agent Service Utilities

This directory contains utility scripts related to interacting with the Coze API.

## Scripts

### `getjwttoken_util.py`

A Python script to generate a JWT token signed with an RSA private key and exchange it for a Coze API access token. This is primarily useful for testing or specific standalone use cases.

**Prerequisites:**

*   Python 3
*   Required packages: `PyJWT`, `cryptography`, `python-dotenv`, `requests`
    ```bash
    pip install PyJWT cryptography python-dotenv requests
    ```

**Setup:**

1.  **Place Keys:** Ensure your RSA private and public keys are placed in the `res/` subdirectory as `.private_key.pem` and `.public_key.pem` respectively. (These files are ignored by git via the root `.gitignore`).
2.  **Create `.env` file:** Create a file named `.env` in this directory (`coze_agent_service/.env`) with your Coze App ID:
    ```dotenv
    # coze_agent_service/.env
    COZE_APP_ID=your_coze_app_id_here
    ```
    *(Replace `your_coze_app_id_here` with your actual App ID. This file is also ignored by git).*

**Usage:**

Navigate to the `coze_agent_service` directory and run the script:

```bash
python getjwttoken_util.py
```

The script will:
1.  Load the `COZE_APP_ID` from `.env`.
2.  Read the private key from `res/.private_key.pem`.
3.  Read the public key from `res/.public_key.pem` (used for the `kid` header).
4.  Generate a JWT token with appropriate claims (`iss`, `aud`, `iat`, `exp`, `jti`).
5.  Print the generated JWT.
6.  Make a POST request to the Coze token endpoint (`https://api.coze.cn/api/permission/oauth2/token`) using the JWT as a bearer token.
7.  Print the JSON response from the Coze API (which should contain the `access_token`, `expires_in`, and `token_type`).

## Directory Structure (`coze_agent_service/`)

```
coze_agent_service/
├── .env              # Local environment variables (ignored by git)
├── README.md         # This file
├── getjwttoken_util.py # The Python script
└── res/              # Resource files
    ├── .private_key.pem # (Ignored by git)
    ├── .public_key.pem  # (Ignored by git)
    └── ...             # Other potential resources
