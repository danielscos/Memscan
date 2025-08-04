#!/bin/bash

# Memscan Runner - Temporary ptrace_scope method
# This script temporarily disables ptrace restrictions to run memscan without sudo
# Supports both GUI and CLI modes

echo "🔧 Memscan Runner - Temporary Security Adjustment"
echo "================================================"

# Parse command line arguments
MODE="gui"
CLI_ARGS=()

for arg in "$@"; do
    case $arg in
        --cli)
            MODE="cli"
            shift
            ;;
        --gui)
            MODE="gui"
            shift
            ;;
        *)
            CLI_ARGS+=("$arg")
            ;;
    esac
done

echo "Running in $MODE mode"
echo

# Check if we're already running as root
if [[ $EUID -eq 0 ]]; then
    echo "❌ Don't run this script as root!"
    echo "   This script is designed to run as a regular user."
    exit 1
fi

# Check if the binaries exist
if [[ "$MODE" == "gui" && ! -f "./target/release/memscan-gui" ]]; then
    echo "❌ Memscan GUI binary not found!"
    echo "   Please build it first with: cargo build --release"
    exit 1
elif [[ "$MODE" == "cli" && ! -f "./target/release/memscan-cli" ]]; then
    echo "❌ Memscan CLI binary not found!"
    echo "   Please build it first with: cargo build --release"
    exit 1
fi

# Check current ptrace scope
PTRACE_SCOPE=$(cat /proc/sys/kernel/yama/ptrace_scope 2>/dev/null || echo "unknown")
echo "📊 Current ptrace_scope: $PTRACE_SCOPE"

case $PTRACE_SCOPE in
    0)
        echo "   ✅ Memory scanning should work without adjustment"
        if [[ "$MODE" == "gui" ]]; then
            echo "   Launching memscan GUI directly..."
            exec ./target/release/memscan-gui
        else
            echo "   Launching memscan CLI directly..."
            exec ./target/release/memscan-cli "${CLI_ARGS[@]}"
        fi
        ;;
    1|2|3)
        echo "   ⚠️  Restricted ptrace - temporarily adjusting security"
        ;;
    *)
        echo "   ❓ Unknown setting - will attempt adjustment"
        ;;
esac

echo
echo "🔒 This script will:"
echo "   1. Temporarily set ptrace_scope=0 (allows memory scanning)"
if [[ "$MODE" == "gui" ]]; then
    echo "   2. Launch memscan GUI as your regular user"
else
    echo "   2. Launch memscan CLI as your regular user"
fi
echo "   3. Restore original ptrace_scope when you exit memscan"
echo
echo "⚠️  WARNING: This temporarily reduces system security!"
echo "   Only processes owned by your user can be scanned."
echo "   System security will be restored when memscan exits."
echo

read -p "Continue? [y/N]: " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Operation cancelled."
    exit 1
fi

echo
echo "🔧 Temporarily adjusting ptrace_scope..."

# Function to restore ptrace_scope on exit
restore_ptrace_scope() {
    echo
    echo "🔒 Restoring original ptrace_scope to $PTRACE_SCOPE..."
    sudo sysctl kernel.yama.ptrace_scope=$PTRACE_SCOPE >/dev/null 2>&1
    if [[ $? -eq 0 ]]; then
        echo "✅ Security settings restored successfully"
    else
        echo "⚠️  Please manually restore with: sudo sysctl kernel.yama.ptrace_scope=$PTRACE_SCOPE"
    fi
}

# Set trap to restore on exit
trap restore_ptrace_scope EXIT

# Temporarily disable ptrace restrictions
echo "Setting ptrace_scope=0..."
sudo sysctl kernel.yama.ptrace_scope=0 >/dev/null 2>&1

if [[ $? -ne 0 ]]; then
    echo "❌ Failed to adjust ptrace_scope. You may need to:"
    echo "   - Check sudo permissions"
    echo "   - Use the regular sudo method instead"
    exit 1
fi

echo "✅ Ptrace restrictions temporarily disabled"
echo

# Verify the change
NEW_SCOPE=$(cat /proc/sys/kernel/yama/ptrace_scope 2>/dev/null || echo "unknown")
if [[ "$NEW_SCOPE" != "0" ]]; then
    echo "❌ Failed to change ptrace_scope (still $NEW_SCOPE)"
    exit 1
fi

if [[ "$MODE" == "gui" ]]; then
    echo "🚀 Launching memscan GUI with temporary privileges..."
    echo "   Memory scanning should now work normally!"
    echo "   Press Ctrl+C or close memscan to restore security settings."
    echo

    # Launch memscan GUI as regular user
    ./target/release/memscan-gui
else
    echo "🚀 Launching memscan CLI with temporary privileges..."
    echo "   Memory scanning should now work normally!"
    echo "   Command: memscan-cli ${CLI_ARGS[*]}"
    echo

    # Launch memscan CLI as regular user
    ./target/release/memscan-cli "${CLI_ARGS[@]}"
fi

# The trap will automatically restore ptrace_scope when this script exits
