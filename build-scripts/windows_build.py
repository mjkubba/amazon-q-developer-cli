#!/usr/bin/env python3

import os
import sys
import subprocess
import argparse
import shutil
from pathlib import Path

def run_command(cmd, cwd=None):
    """Run a command and return its output"""
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error: {result.stderr}")
        sys.exit(1)
    return result.stdout.strip()

def build_project(release=False, target=None):
    """Build the project"""
    cmd = ["cargo", "build"]
    
    if release:
        cmd.append("--release")
    
    if target:
        cmd.extend(["--target", target])
    
    run_command(cmd)
    print("Build completed successfully")

def create_installer(version):
    """Create Windows installer"""
    # This is a placeholder for actual installer creation
    # In a real implementation, this would use WiX Toolset or similar
    print(f"Creating installer for version {version}")
    
    # Example WiX command (not actually run)
    # run_command(["candle", "installer.wxs"])
    # run_command(["light", "-ext", "WixUIExtension", "installer.wixobj", "-out", f"AmazonQ-{version}.msi"])
    
    print("Installer creation placeholder - not actually creating installer")

def main():
    parser = argparse.ArgumentParser(description="Build Amazon Q CLI for Windows")
    parser.add_argument("--release", action="store_true", help="Build in release mode")
    parser.add_argument("--target", default="x86_64-pc-windows-msvc", help="Build target")
    parser.add_argument("--create-installer", action="store_true", help="Create installer")
    parser.add_argument("--version", default="0.1.0", help="Version for installer")
    
    args = parser.parse_args()
    
    # Build the project
    build_project(release=args.release, target=args.target)
    
    # Create installer if requested
    if args.create_installer:
        create_installer(args.version)

if __name__ == "__main__":
    main()
