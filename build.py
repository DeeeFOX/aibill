#!/usr/bin/env python3
import argparse
import subprocess
import sys
import platform
import re
import os
import json
from getpass import getpass

def get_alicloud_config():
    """Get AliCloud container registry config"""
    # Try environment variables first
    registry = os.getenv('ALICLOUD_REGISTRY')
    namespace = os.getenv('ALICLOUD_NAMESPACE')
    username = os.getenv('ALICLOUD_USERNAME')
    
    # Fallback to config file
    if not all([registry, namespace, username]):
        try:
            with open('.build_config', 'r') as f:
                config = json.load(f)
            registry = registry or config.get('registry')
            namespace = namespace or config.get('namespace')
            username = username or config.get('username')
        except (FileNotFoundError, json.JSONDecodeError):
            pass
            
    password = os.getenv('ALICLOUD_PASSWORD') or getpass('AliCloud registry password: ')
    return registry, namespace, username, password

def get_version():
    """Read version from Cargo.toml"""
    cargo_path = os.path.join(os.path.dirname(__file__), 'coze_token_service/Cargo.toml')
    with open(cargo_path, 'r') as f:
        content = f.read()
    match = re.search(r'version\s*=\s*"([^"]+)"', content)
    if match:
        return match.group(1)
    return 'latest'

def main():
    parser = argparse.ArgumentParser(description='Docker build script with architecture selection')
    parser.add_argument('arch', choices=['x86', 'arm64'], 
                       help='Target architecture (x86 for Intel, arm64 for Apple Silicon)')
    args = parser.parse_args()
    version = get_version()

    # Map architecture to docker platform
    platform_map = {
        'x86': 'linux/amd64',
        'arm64': 'linux/arm64'
    }

    # Get current directory as build context
    build_context = '.'
    # use uname get build paltform info
    build_paltform = subprocess.check_output(['uname', '-m']).decode().strip()
    if build_paltform in ('x86_64', 'amd64'):
        build_paltform = 'linux/amd64'
    elif build_paltform in ('aarch64', 'arm64'):
        build_paltform = 'linux/arm64'
    else:
        print(f"Unsupported build platform: {build_paltform}", file=sys.stderr)
        sys.exit(1)
    try:
        target = "x86_64-unknown-linux-musl" if args.arch == "x86" else "aarch64-unknown-linux-musl"
        platform = platform_map[args.arch]
        cmd = [
            'docker', 'buildx', 'build',
            '--platform', platform,
            '--build-arg', f'BUILDPLATFORM={build_paltform}',
            '--build-arg', f'TARGETPLATFORM={platform}',
            '--build-arg', f'TARGET={target}',
            '--progress', 'plain',
            '-t', f'coze_token_service:{version}',
            '-t', 'coze_token_service:latest',
            build_context
        ]
        print(f"Running command: {' '.join(cmd)}")
        print(f"Building Docker image for {args.arch} architecture...")
        subprocess.run(cmd, check=True)
        print("Build completed successfully")
        
        # Push to AliCloud registry if configured
        registry, namespace, username, password = get_alicloud_config()
        if registry and namespace:
            print(f"Pushing images to AliCloud registry {registry}...")
            try:
                # Login to registry
                subprocess.run([
                    'docker', 'login',
                    '--username', username,
                    '--password-stdin',
                    registry
                ], input=password.encode(), check=True)
                
                # Tag and push versioned image
                remote_image = f"{registry}/{namespace}/coze_token_service:{version}"
                subprocess.run([
                    'docker', 'tag',
                    f'coze_token_service:{version}',
                    remote_image
                ], check=True)
                subprocess.run(['docker', 'push', remote_image], check=True)
                
                # Tag and push latest image
                remote_latest = f"{registry}/{namespace}/coze_token_service:latest"
                subprocess.run([
                    'docker', 'tag',
                    'coze_token_service:latest',
                    remote_latest
                ], check=True)
                subprocess.run(['docker', 'push', remote_latest], check=True)
                
                print(f"Images pushed successfully to {registry}")
            except subprocess.CalledProcessError as e:
                print(f"Push failed with error: {e}", file=sys.stderr)
                
    except subprocess.CalledProcessError as e:
        print(f"Build failed with error: {e}", file=sys.stderr)
        sys.exit(1)
    except KeyError:
        print(f"Invalid architecture: {args.arch}", file=sys.stderr)
        sys.exit(1)

if __name__ == '__main__':
    main()
