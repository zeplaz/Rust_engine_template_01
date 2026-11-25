@echo off
setlocal EnableExtensions
rem Windows stdio launcher for Cursor MCP (BUILD-READ / designer-mcp lane)
cd /d "%~dp0..\python"
if not defined RUST_ENGINE_PYTHON (
  set "RUST_ENGINE_PYTHON=python"
)
if not defined RUST_ENGINE_REPO (
  set "RUST_ENGINE_REPO=%~dp0..\..\.."
)
set PYTHONUNBUFFERED=1
set PYTHONIOENCODING=utf-8
set PYTHONUTF8=1
"%RUST_ENGINE_PYTHON%" -u -m rust_engine_mcp.server
