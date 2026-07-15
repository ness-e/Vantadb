"""
VantaOutputValidator for verifying VantaDB Campaign Executor output.

Validates shell commands, file paths, generated code, and general
agent output for safety and correctness before execution.
"""

from typing import Dict, List, Any, Optional, NamedTuple
from pathlib import Path
import ast
import importlib.util
import os
import sys
import time


class ValidationResult(NamedTuple):
    valid: bool
    risk_level: str  # "safe", "moderate", "dangerous"
    errors: List[str]
    warnings: List[str]
    checks_passed: List[str]


SAFE_RESULT = ValidationResult(valid=True, risk_level="safe", errors=[], warnings=[], checks_passed=[])

DANGEROUS_COMMAND_PATTERNS = [
    "rm -rf",
    "rm -fr",
    "rm -r /",
    "rm -f /",
    "del /f",
    "rd /s /q",
    "format ",
    ":(){" " :(){ :|:& };:",  # fork bomb
    "> /dev/null",
    "> /dev/sda",
    "> /dev/sdb",
    "mkfs.",
    "dd if=",
    ":(){",
    "chmod 777 /",
    "chown ",
]

DANGEROUS_PIPED_COMMANDS = [
    "| bash",
    "| sh",
    "| zsh",
    "| pwsh",
    "| powershell",
    "| cmd",
    "`",
    "$(",
]

SYSTEM_DIRECTORIES = [
    "/etc", "/bin", "/sbin", "/usr", "/boot", "/dev", "/proc", "/sys",
    "C:\\Windows", "C:\\System32", "C:\\Program Files",
    "/System", "/Library", "/Applications",
]

ALLOWED_PYTHON_IMPORTS = [
    "json", "os", "pathlib", "typing", "collections", "datetime",
    "re", "math", "random", "itertools", "functools", "copy",
    "enum", "dataclasses", "abc", "hashlib", "uuid",
]


class VantaOutputValidator:
    """
    Validates VantaDB Campaign Executor outputs (commands, paths, code).
    """

    def __init__(self, workspace_dir: Optional[str] = None):
        self.workspace_dir = Path(workspace_dir or Path.cwd()).resolve()
        self.validation_results: List[Dict[str, Any]] = []

    def validate_shell_command(
        self,
        command: str,
        allowed_commands: Optional[List[str]] = None,
    ) -> ValidationResult:
        errors: List[str] = []
        warnings: List[str] = []
        checks: List[str] = []

        if not command or not command.strip():
            errors.append("Empty command")
            return ValidationResult(valid=False, risk_level="dangerous", errors=errors, warnings=[], checks_passed=[])

        cmd_lower = command.strip().lower()

        cmd_name = command.strip().split()[0] if command.strip().split() else ""
        if allowed_commands and cmd_name not in allowed_commands:
            errors.append(f"Command '{cmd_name}' is not in allowed_commands list")
            return ValidationResult(valid=False, risk_level="dangerous", errors=errors, warnings=[], checks_passed=[])

        for pattern in DANGEROUS_COMMAND_PATTERNS:
            if pattern.lower() in cmd_lower:
                errors.append(f"Command contains dangerous pattern: {pattern}")
                return ValidationResult(valid=False, risk_level="dangerous", errors=errors, warnings=[], checks_passed=[])

        for pattern in DANGEROUS_PIPED_COMMANDS:
            if pattern in command:
                warnings.append(f"Command pipes to shell ({pattern}) — potential injection risk")

        checks.append("Shell command passed safety checks")
        risk_level = "dangerous" if errors else ("moderate" if warnings else "safe")
        return ValidationResult(valid=len(errors) == 0, risk_level=risk_level, errors=errors, warnings=warnings, checks_passed=checks)

    def validate_file_path(self, path: str) -> ValidationResult:
        errors: List[str] = []
        warnings: List[str] = []
        checks: List[str] = []

        if not path or not path.strip():
            errors.append("Empty path")
            return ValidationResult(valid=False, risk_level="dangerous", errors=errors, warnings=[], checks_passed=[])

        resolved = Path(path).resolve()

        if ".." in path.split(os.sep) or "../" in path.replace("\\", "/"):
            errors.append(f"Path contains parent directory traversal: {path}")
            return ValidationResult(valid=False, risk_level="dangerous", errors=errors, warnings=[], checks_passed=[])

        try:
            resolved.relative_to(self.workspace_dir)
            checks.append("Path is within workspace directory")
        except ValueError:
            errors.append(f"Path '{path}' escapes workspace directory '{self.workspace_dir}'")
            return ValidationResult(valid=False, risk_level="dangerous", errors=errors, warnings=[], checks_passed=[])

        for sys_dir in SYSTEM_DIRECTORIES:
            if str(resolved).lower().startswith(sys_dir.lower()):
                errors.append(f"Path writes to system directory: {sys_dir}")
                return ValidationResult(valid=False, risk_level="dangerous", errors=errors, warnings=[], checks_passed=[])

        checks.append("File path passed safety checks")
        risk_level = "dangerous" if errors else ("moderate" if warnings else "safe")
        return ValidationResult(valid=len(errors) == 0, risk_level=risk_level, errors=errors, warnings=warnings, checks_passed=checks)

    def validate_output(self, output: str, output_type: str = "text") -> ValidationResult:
        errors: List[str] = []
        warnings: List[str] = []
        checks: List[str] = []

        if not output or not output.strip():
            errors.append("Empty output")
            return ValidationResult(valid=False, risk_level="dangerous", errors=errors, warnings=[], checks_passed=[])

        if output_type == "shell":
            return self.validate_shell_command(output)

        if output_type == "file_path":
            return self.validate_file_path(output)

        if output_type in ("python", "code"):
            for dangerous in ["import os", "import subprocess", "import sys", "eval(", "exec(", "__import__("]:
                if dangerous in output:
                    warnings.append(f"Generated code contains: {dangerous}")

            try:
                ast.parse(output)
                checks.append("Valid Python syntax")
            except SyntaxError as e:
                errors.append(f"Python syntax error: {e}")
                return ValidationResult(valid=False, risk_level="dangerous", errors=errors, warnings=warnings, checks_passed=checks)

            tree = ast.parse(output)
            imports = []
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        imports.append(alias.name.split(".")[0])
                elif isinstance(node, ast.ImportFrom):
                    if node.module:
                        imports.append(node.module.split(".")[0])

            for mod in imports:
                if mod not in ALLOWED_PYTHON_IMPORTS and mod != "__future__":
                    warnings.append(f"Import '{mod}' is outside the allowed set — verify it is safe")

            checks.append("Output passed validation")
            risk_level = "dangerous" if errors else ("moderate" if warnings else "safe")
            return ValidationResult(valid=len(errors) == 0, risk_level=risk_level, errors=errors, warnings=warnings, checks_passed=checks)

        if output_type == "sql":
            dangerous_keywords = ["drop ", "truncate ", "alter ", "create ", "grant ", "revoke "]
            kw_lower = output.lower()
            for kw in dangerous_keywords:
                if kw in kw_lower:
                    warnings.append(f"SQL contains DDL/DCL keyword: {kw.strip()}")
            checks.append("SQL output validated")
            risk_level = "dangerous" if errors else ("moderate" if warnings else "safe")
            return ValidationResult(valid=len(errors) == 0, risk_level=risk_level, errors=errors, warnings=warnings, checks_passed=checks)

        if output_type == "html":
            warnings.append("HTML output — review for XSS before serving")

        checks.append("Output validated")
        return ValidationResult(valid=True, risk_level="moderate" if warnings else "safe", errors=errors, warnings=warnings, checks_passed=checks)

    def validate_agent(self, agent_path: str) -> Dict[str, Any]:
        results = {
            'valid': True,
            'errors': [],
            'warnings': [],
            'checks_passed': [],
            'agent_info': {},
        }

        try:
            structure_valid = self._validate_structure(agent_path, results)
            if not structure_valid:
                results['valid'] = False
                return results

            syntax_valid = self._validate_syntax(agent_path, results)
            if not syntax_valid:
                results['valid'] = False
                return results

            impl_valid = self._validate_implementation(agent_path, results)
            if not impl_valid:
                results['valid'] = False

            self._validate_dependencies(agent_path, results)

        except Exception as e:
            results['valid'] = False
            results['errors'].append(f"Validation failed: {str(e)}")

        return results

    def _validate_structure(self, agent_path: str, results: Dict[str, Any]) -> bool:
        path = Path(agent_path)

        if path.is_file():
            if path.suffix != '.py':
                results['errors'].append("Agent file must be a Python file (.py)")
                return False
            results['checks_passed'].append("Valid Python file")
            results['agent_info']['type'] = 'single_file'
            results['agent_info']['main_file'] = str(path)
        else:
            agent_dir = path / "agent"
            if not agent_dir.exists():
                results['errors'].append("Agent directory must contain 'agent' subdirectory")
                return False
            agent_file = agent_dir / "agent.py"
            if not agent_file.exists():
                results['errors'].append("Agent directory must contain agent/agent.py")
                return False
            results['checks_passed'].append("Valid directory structure")
            results['agent_info']['type'] = 'directory'
            results['agent_info']['main_file'] = str(agent_file)

        return True

    def _validate_syntax(self, agent_path: str, results: Dict[str, Any]) -> bool:
        main_file = results['agent_info'].get('main_file')
        if not main_file:
            results['errors'].append("No main file found")
            return False

        try:
            with open(main_file, 'r') as f:
                content = f.read()

            tree = ast.parse(content)
            results['checks_passed'].append("Valid Python syntax")

            classes = [node for node in ast.walk(tree) if isinstance(node, ast.ClassDef)]
            if not classes:
                results['warnings'].append("No class definitions found")
            else:
                results['agent_info']['classes'] = [cls.name for cls in classes]

                agent_classes = [cls for cls in classes if 'Agent' in cls.name]
                agent_classes.sort(
                    key=lambda c: (c.name != 'Agent', not c.name.endswith('Agent'))
                )
                if agent_classes:
                    results['agent_info']['agent_class'] = agent_classes[0].name
                else:
                    results['warnings'].append("No class with 'Agent' in name found")

            return True

        except SyntaxError as e:
            results['errors'].append(f"Syntax error: {str(e)}")
            return False
        except Exception as e:
            results['errors'].append(f"Failed to parse file: {str(e)}")
            return False

    def _validate_implementation(self, agent_path: str, results: Dict[str, Any]) -> bool:
        main_file = results['agent_info'].get('main_file')
        agent_class_name = results['agent_info'].get('agent_class')

        if not main_file:
            results['errors'].append("No main agent file resolved; cannot validate implementation")
            return False
        if not agent_class_name:
            results['errors'].append("No class with 'Agent' in name found in file")
            return False

        file_path = Path(main_file)

        if not file_path.exists():
            results['errors'].append(f"Agent file does not exist: {main_file}")
            return False

        try:
            source = file_path.read_text(encoding='utf-8')
            tree = ast.parse(source)
        except SyntaxError as e:
            results['errors'].append(f"Agent file has syntax errors: {e}")
            return False
        except Exception as e:
            results['errors'].append(f"Failed to read agent file: {e}")
            return False

        agent_classes = [
            node for node in ast.walk(tree)
            if isinstance(node, ast.ClassDef) and 'Agent' in node.name
        ]
        if not agent_classes:
            results['errors'].append("No class with 'Agent' in name found in file")
            return False

        agent_classes.sort(
            key=lambda c: (c.name != 'Agent', not c.name.endswith('Agent'))
        )
        target_class = agent_classes[0]
        defined_methods = {
            node.name
            for node in ast.walk(target_class)
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        }

        required_methods = ['solve_task', '__init__']
        missing = []
        for method_name in required_methods:
            if method_name in defined_methods:
                results['checks_passed'].append(f"Has method: {method_name}")
            else:
                missing.append(method_name)
                results['errors'].append(f"Missing required method: {method_name}")

        if missing:
            return False

        unique_name = f"_vantadb_agent_validate_{id(file_path)}_{int(time.time() * 1e6)}"
        try:
            spec = importlib.util.spec_from_file_location(unique_name, file_path)
            if spec is None or spec.loader is None:
                results['warnings'].append("Cannot create module spec — skipping runtime load check")
                results['checks_passed'].append("Agent class can be referenced (AST only)")
                return True

            module = importlib.util.module_from_spec(spec)
            sys.modules[unique_name] = module
            try:
                spec.loader.exec_module(module)
            finally:
                sys.modules.pop(unique_name, None)

            agent_class = getattr(module, agent_class_name, None)
            if agent_class is None:
                for attr_name in dir(module):
                    if 'Agent' in attr_name:
                        agent_class = getattr(module, attr_name)
                        break

            if agent_class is None:
                results['errors'].append(f"Class '{agent_class_name}' not found after loading {main_file}")
                return False

            for method_name in required_methods:
                if not hasattr(agent_class, method_name):
                    results['errors'].append(f"Loaded class missing required method: {method_name}")
                    return False

            import inspect
            if hasattr(agent_class, 'solve_task'):
                sig = inspect.signature(agent_class.solve_task)
                params = list(sig.parameters.keys())
                if 'task' not in params:
                    results['warnings'].append("solve_task should have 'task' parameter")

            results['checks_passed'].append("Agent class loaded and verified successfully")
            return True

        except Exception as e:
            results['errors'].append(f"Implementation validation failed: {str(e)}")
            return False

    def _validate_dependencies(self, agent_path: str, results: Dict[str, Any]) -> None:
        main_file = results['agent_info'].get('main_file')
        if not main_file:
            return

        try:
            with open(main_file, 'r') as f:
                content = f.read()

            expected_imports = [
                ('vantadb_interface', 'VantaDB interface'),
                ('tools', 'Tool system'),
                ('asyncio', 'Async support'),
            ]

            for module, description in expected_imports:
                if f"import {module}" in content or f"from {module}" in content:
                    results['checks_passed'].append(f"Uses {description}")
                else:
                    results['warnings'].append(f"Does not import {module} ({description})")

        except Exception as e:
            results['warnings'].append(f"Failed to check dependencies: {str(e)}")

    def get_validation_summary(self, results: Dict[str, Any]) -> str:
        lines = ["VantaOutput Validation Summary", "=" * 50]

        if results.get('valid'):
            lines.append("VALID output")
        else:
            lines.append("VALIDATION FAILED")

        if results.get('agent_info'):
            lines.append(f"\n  Type: {results['agent_info'].get('type', 'unknown')}")
            if 'agent_class' in results['agent_info']:
                lines.append(f"  Class: {results['agent_info']['agent_class']}")

        if results.get('checks_passed'):
            lines.append(f"\n  Passed {len(results['checks_passed'])}:")
            for check in results['checks_passed']:
                lines.append(f"    - {check}")

        if results.get('errors'):
            lines.append(f"\n  Errors ({len(results['errors'])}):")
            for error in results['errors']:
                lines.append(f"    - {error}")

        if results.get('warnings'):
            lines.append(f"\n  Warnings ({len(results['warnings'])}):")
            for warning in results['warnings']:
                lines.append(f"    - {warning}")

        return "\n".join(lines)

    def log_validation(self, result: ValidationResult, context: str = "") -> None:
        self.validation_results.append({
            "context": context,
            "valid": result.valid,
            "risk_level": result.risk_level,
            "errors": result.errors,
            "warnings": result.warnings,
        })

    def get_validation_history(self) -> List[Dict[str, Any]]:
        return list(self.validation_results)
