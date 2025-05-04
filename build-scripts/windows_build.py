#!/usr/bin/env python3
"""
Windows build script for Amazon Q Developer CLI.
This script builds the project for Windows and optionally creates an installer.
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path

def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Build Amazon Q Developer CLI for Windows")
    parser.add_argument("--release", action="store_true", help="Build in release mode")
    parser.add_argument("--target", default="x86_64-pc-windows-msvc", 
                        help="Target triple (default: x86_64-pc-windows-msvc)")
    parser.add_argument("--create-installer", action="store_true", help="Create Windows installer")
    parser.add_argument("--version", default="0.1.0", help="Version number for the installer")
    parser.add_argument("--no-default-features", action="store_true", help="Build without default features")
    parser.add_argument("--features", help="Comma-separated list of features to enable")
    return parser.parse_args()

def run_command(cmd, cwd=None):
    """Run a command and return its output."""
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Command failed with exit code {result.returncode}")
        print(f"STDOUT: {result.stdout}")
        print(f"STDERR: {result.stderr}")
        sys.exit(result.returncode)
    return result.stdout

def build_project(args):
    """Build the project."""
    # First try building just the q_cli crate with specific features
    print("Building q_cli crate...")
    q_cli_cmd = ["cargo", "build", "-p", "q_cli"]
    
    if args.release:
        q_cli_cmd.append("--release")
    
    q_cli_cmd.extend(["--target", args.target])
    q_cli_cmd.append("--no-default-features")
    q_cli_cmd.extend(["--features", "windows-terminal,minimal"])
    
    try:
        run_command(q_cli_cmd)
    except SystemExit:
        print("Failed to build q_cli with Windows features. Trying with minimal features...")
        q_cli_cmd = ["cargo", "build", "-p", "q_cli"]
        if args.release:
            q_cli_cmd.append("--release")
        q_cli_cmd.extend(["--target", args.target])
        q_cli_cmd.append("--no-default-features")
        q_cli_cmd.extend(["--features", "minimal"])
        run_command(q_cli_cmd)
    
    # Then try building the entire project
    print("Building entire project...")
    cargo_cmd = ["cargo", "build"]
    if args.release:
        cargo_cmd.append("--release")
    
    cargo_cmd.extend(["--target", args.target])
    
    if args.no_default_features:
        cargo_cmd.append("--no-default-features")
    
    if args.features:
        cargo_cmd.extend(["--features", args.features])
    else:
        cargo_cmd.extend(["--features", "windows-terminal,minimal"])
    
    try:
        run_command(cargo_cmd)
    except SystemExit:
        print("Failed to build entire project. Some components may not be compatible with Windows.")
    
    return 0

def create_installer(args):
    """Create a Windows installer using WiX Toolset."""
    print("Creating Windows installer...")
    
    # This is a placeholder for the actual installer creation code
    # In a real implementation, this would use WiX Toolset or similar
    
    # Check if WiX Toolset is installed
    try:
        run_command(["candle", "--version"])
    except FileNotFoundError:
        print("WiX Toolset not found. Please install it from https://wixtoolset.org/")
        return 1
    
    # Create WiX source file
    wix_file = Path("installer/amazon-q.wxs")
    wix_file.parent.mkdir(exist_ok=True)
    
    # Create a basic WiX file (this is just a placeholder)
    wix_content = f"""<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="*" Name="Amazon Q Developer CLI" Language="1033" 
             Version="{args.version}" Manufacturer="Amazon Web Services" 
             UpgradeCode="12345678-1234-1234-1234-123456789012">
        <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
        <MajorUpgrade DowngradeErrorMessage="A newer version of Amazon Q Developer CLI is already installed." />
        <MediaTemplate EmbedCab="yes" />
        <Feature Id="ProductFeature" Title="Amazon Q Developer CLI" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
        </Feature>
    </Product>
    <Fragment>
        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFilesFolder">
                <Directory Id="INSTALLFOLDER" Name="Amazon Q Developer CLI" />
            </Directory>
        </Directory>
    </Fragment>
    <Fragment>
        <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
            <Component Id="ProductComponent">
                <File Source="target/{args.target}/release/q.exe" />
            </Component>
        </ComponentGroup>
    </Fragment>
</Wix>
"""
    
    with open(wix_file, "w") as f:
        f.write(wix_content)
    
    # Compile and link the installer
    run_command(["candle", str(wix_file)])
    run_command(["light", "-ext", "WixUIExtension", f"{wix_file.stem}.wixobj", "-o", f"Amazon-Q-Developer-CLI-{args.version}.msi"])
    
    print(f"Installer created: Amazon-Q-Developer-CLI-{args.version}.msi")
    return 0

def main():
    """Main entry point."""
    args = parse_args()
    
    # Build the project
    build_project(args)
    
    # Create installer if requested
    if args.create_installer:
        create_installer(args)
    
    return 0

if __name__ == "__main__":
    sys.exit(main())
