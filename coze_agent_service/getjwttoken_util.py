# You must run `pip install PyJWT cryptography` to install the PyJWT and the cryptography packages in order to use this script.

#!/usr/bin/env python3
import os
from pathlib import Path
import sys
import time
import uuid

from dotenv import load_dotenv
import jwt
import requests

# 替换为你的实际 Coze App 私钥
# 从当前目录的private_key.pem文件中读取私钥
signing_key = '''
-----BEGIN PRIVATE KEY-----
xxxxxxxxxxxxxxxxxx
-----END PRIVATE KEY-----
'''
with open('./res/.private_key.pem', 'r') as f:
    signing_key = f.read()


# 从./config/.env文件获取Coze App ID作为环境变量，并从环境变量中加载Coze App ID到变量中
env_path = Path(__file__).parent / '.env'
coze_app_id = os.getenv('COZE_APP_ID')
print(coze_app_id)

payload = {
    'iat': int(time.time()),
    'exp': int(time.time()) + 86400,
    "jti": str(uuid.uuid4()),
    'aud': 'api.coze.cn',   # 替换为实际的coze api domain
    'iss': coze_app_id  # 替换为你的实际 Coze App ID
}

with open("./res/.public_key.pem", "r") as f:
    public_key = f.read()

headers = {
    'kid': public_key  # 替换为你的实际 Coze App 公钥指纹
}

# Create JWT with headers
encoded_jwt = jwt.encode(payload, signing_key,
                         algorithm='RS256', headers=headers)

print(f"JWT: {encoded_jwt}")


# curl --location 'https://api.coze.cn/api/permission/oauth2/token' \
# --header 'Content-Type: application/json' \
# --header 'Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6InZZd2ZsdFR1OWZBbWtwWFhSdnR5UmREc3RONVMzZWNFcDFqVzB6dVQyRE****.eyJpc3MiOiIzMTAwMDAwMDAwMDIiLCJhdWQiOiJhcGkuY296ZS5jb20iLCJpYXQiOjE1MTYyMzkwMjIsImV4cCI6MTkxNjI1OTAyMiwianRpIjoiZmhqaGFsc2tqZmFkc2pld3F****.CuoiCCF-nHFyGmu2EKlwFoyd3uDyKQ3Drc1CrXQyMVySTzZlZd2M7zKWsziB3AktwbUZiRJlQ1HbghR05CW2YRHwKL4-dlJ4koR3onU7iQAO5DkPCaIxbAuTsQobtCAdkkZTg8gav9EnN1QN_1xq0w8BzuuhS7wCeY8UbaskkTK9GnO4eU9tEINmVw-2CrfB-kNbEHlEDwXfcrb4YPpkw3GhmuPShenNLObfSWS0CqIyakXL8qD5AgXLoB-SejAsRdzloSUInNXENJHfSVMkThxRhJy7yEjX3BmculC54fMKENRfLElBqwJyLLUjeRHsYnaru2ca4W8_yaPJ7F****' \
# --data '{
#     "duration_seconds": 86399,
#     "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer"
# }'

res = requests.post('https://api.coze.cn/api/permission/oauth2/token',
                    headers={
                        'Content-Type': 'application/json',
                        'Authorization': f'Bearer {encoded_jwt}'
                    },
                    json={
                        "duration_seconds": 86399,
                        "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer"
                    }
                    )

print(res.json())
