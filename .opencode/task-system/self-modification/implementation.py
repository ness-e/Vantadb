"""
Implementation module for applying pipeline modifications.

This module provides functionality to implement modification proposals,
including backup creation, pipeline changes, and verification.
"""

import ast
import importlib.util
import shutil
import tempfile
import datetime
import asyncio
import traceback
from typing import Dict, List, Any, Optional
from pathlib import Path


class PipelineUpdater:
    """
    Manages the implementation of pipeline modifications.

    This class handles the actual application of proposed changes,
    including safety measures like backups and verification.
    """

    def __init__(self):
        """Initialize the pipeline updater."""
        self.backup_dir = None
        self.changes_applied = []
        self.verification_results = []

    async def implement_proposal(
        self,
        proposal: 'PipelineProposal',
        pipeline_path: str,
        dry_run: bool = False
    ) -> Dict[str, Any]:
        """
        Implement a modification proposal.

        Args:
            proposal: The modification proposal to implement
            pipeline_path: Path to the pipeline code
            dry_run: If True, simulate changes without applying

        Returns:
            Dict containing implementation results
        """
        results = {
            'success': False,
            'changes_applied': [],
            'errors': [],
            'backup_path': None,
            'verification': None
        }

        try:
            # Create backup unless dry run
            if not dry_run:
                self.backup_dir = self._create_backup(pipeline_path)
                results['backup_path'] = str(self.backup_dir)

            # Apply each change
            for change in proposal.code_changes:
                try:
                    success = await self._apply_pipeline_change(
                        change,
                        pipeline_path,
                        dry_run
                    )
                    if success:
                        results['changes_applied'].append({
                            'file': change.file_path,
                            'type': change.change_type,
                            'description': change.description
                        })
                        self.changes_applied.append(change)
                except Exception as e:
                    error_msg = f"Failed to apply change '{change.description}': {str(e)}"
                    results['errors'].append(error_msg)

                    # Rollback on error unless dry run
                    if not dry_run and self.backup_dir:
                        self._rollback_changes(pipeline_path)
                        results['success'] = False
                        return results

            # Verify changes unless dry run
            if not dry_run:
                verification = await self._verify_modifications(pipeline_path)
                results['verification'] = verification

                if not verification['valid']:
                    # Rollback if verification fails
                    self._rollback_changes(pipeline_path)
                    results['errors'].extend(verification['errors'])
                    results['success'] = False
                    return results

            results['success'] = True

        except Exception as e:
            results['errors'].append(f"Implementation failed: {str(e)}")
            if not dry_run and self.backup_dir:
                self._rollback_changes(pipeline_path)

        return results

    def _create_backup(self, pipeline_path: str) -> Path:
        """
        Create a backup of the pipeline code.

        Args:
            pipeline_path: Path to pipeline code

        Returns:
            Path to backup directory
        """
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_path = Path(tempfile.mkdtemp(prefix=f"pipeline_backup_{timestamp}_"))

        # Copy entire pipeline directory
        src_path = Path(pipeline_path)
        if src_path.exists():
            shutil.copytree(src_path, backup_path / "pipeline", dirs_exist_ok=True)

        return backup_path

    async def _apply_pipeline_change(
        self,
        change: 'PipelineChange',
        pipeline_path: str,
        dry_run: bool
    ) -> bool:
        """
        Apply a single pipeline change.

        Args:
            change: The pipeline change to apply
            pipeline_path: Path to pipeline code
            dry_run: If True, simulate without applying

        Returns:
            bool: True if successful
        """
        file_path = Path(pipeline_path) / change.file_path

        if dry_run:
            # In dry run mode, simulate success for all valid operations
            if change.change_type == 'add':
                return True
            elif change.change_type in ['modify', 'delete'] and file_path.exists():
                return True
            elif change.change_type in ['modify', 'delete'] and not file_path.exists():
                return False
            return True

        # Ensure directory exists
        file_path.parent.mkdir(parents=True, exist_ok=True)

        if change.change_type == 'add':
            return self._apply_add_change(file_path, change)
        elif change.change_type == 'modify':
            return self._apply_modify_change(file_path, change)
        elif change.change_type == 'delete':
            return self._apply_delete_change(file_path, change)

        return False

    def _apply_add_change(self, file_path: Path, change: 'PipelineChange') -> bool:
        """Apply an 'add' type change."""
        if not file_path.exists():
            # Create new file
            if change.new_code:
                file_path.write_text(change.new_code)
                return True
        else:
            # Add to existing file
            content = file_path.read_text()

            if change.location == 'config' and change.new_code:
                # Add config at the top
                lines = content.split('\n')
                insert_line = None
                for i, line in enumerate(lines):
                    if line.strip() and not line.startswith('#') and not line.startswith('"""'):
                        insert_line = i
                        break

                if insert_line is not None:
                    lines.insert(insert_line, change.new_code)
                    file_path.write_text('\n'.join(lines))
                    return True

            elif change.location and change.new_code:
                # Add under specific section
                lines = content.split('\n')
                insert_line = None
                for i, line in enumerate(lines):
                    if change.location in line.lower():
                        insert_line = i + 1
                        break

                if insert_line:
                    lines.insert(insert_line, change.new_code)
                    file_path.write_text('\n'.join(lines))
                    return True

            else:
                # Default: append to end
                content += f"\n\n{change.new_code}"
                file_path.write_text(content)
                return True

        return False

    def _apply_modify_change(self, file_path: Path, change: 'PipelineChange') -> bool:
        """
        Apply a 'modify' type change.

        Requires exactly one occurrence of ``change.old_code`` in the file.
        Zero occurrences → raises RuntimeError (old_code not found).
        More than one → raises RuntimeError (ambiguous match).
        """
        if not file_path.exists():
            return False

        if not (change.old_code and change.new_code):
            return False

        content = file_path.read_text()
        occurrences = content.count(change.old_code)

        if occurrences == 0:
            raise RuntimeError(
                f"old_code not found in {file_path}: no occurrences of the search text"
            )
        if occurrences > 1:
            raise RuntimeError(
                f"Ambiguous match in {file_path}: {occurrences} occurrences found; "
                "provide more context to make the match unique"
            )

        new_content = content.replace(change.old_code, change.new_code, 1)
        file_path.write_text(new_content)
        return True

    def _apply_delete_change(self, file_path: Path, change: 'PipelineChange') -> bool:
        """Apply a 'delete' type change."""
        if file_path.exists():
            if change.old_code:
                content = file_path.read_text()
                if change.old_code in content:
                    content = content.replace(change.old_code, '')
                    file_path.write_text(content)
                    return True
            else:
                file_path.unlink()
                return True

        return False

    def _rollback_changes(self, pipeline_path: str) -> None:
        """
        Rollback changes using backup.

        Args:
            pipeline_path: Path to pipeline code
        """
        if not self.backup_dir:
            return

        # Remove current pipeline directory
        pipeline_dir = Path(pipeline_path)
        if pipeline_dir.exists():
            shutil.rmtree(pipeline_dir)

        # Restore from backup
        backup_pipeline_dir = self.backup_dir / "pipeline"
        if backup_pipeline_dir.exists():
            shutil.copytree(backup_pipeline_dir, pipeline_dir)

    async def _verify_modifications(self, pipeline_path: str) -> Dict[str, Any]:
        """
        Verify that modifications are valid.

        Args:
            pipeline_path: Path to pipeline code

        Returns:
            Dict with verification results
        """
        results = {
            'valid': True,
            'errors': [],
            'warnings': []
        }

        # Check Python syntax
        pipeline_files = Path(pipeline_path).rglob("*.py")
        for file_path in pipeline_files:
            try:
                content = file_path.read_text()
                ast.parse(content)
            except SyntaxError as e:
                results['valid'] = False
                results['errors'].append(f"Syntax error in {file_path}: {str(e)}")
            except Exception as e:
                results['warnings'].append(f"Failed to parse {file_path}: {str(e)}")

        # Check imports
        try:
            import_errors = self._check_imports(pipeline_path)
            if import_errors:
                results['errors'].extend(import_errors)
                results['valid'] = False
        except Exception as e:
            results['warnings'].append(f"Import verification failed: {str(e)}")

        # Check YAML files for basic validity
        yaml_files = Path(pipeline_path).rglob("*.yaml")
        for file_path in yaml_files:
            try:
                content = file_path.read_text()
                # Basic check: ensure at least one key-value pair exists
                if ':' not in content:
                    results['warnings'].append(
                        f"{file_path}: YAML file may be malformed (no key-value pairs)"
                    )
            except Exception as e:
                results['warnings'].append(f"Failed to read {file_path}: {str(e)}")

        return results

    def _check_imports(self, pipeline_path: str) -> List[str]:
        """
        Check that every top-level import in modified Python files can be resolved.

        For each ``.py`` file under *pipeline_path* the function:
        1. Parses the source with :mod:`ast` to extract top-level ``import``
           and ``from … import`` statements.
        2. Calls :func:`importlib.util.find_spec` for each top-level module
           name to verify it is resolvable.

        Relative imports (``from . import …``) are skipped because they
        require a package context.

        Args:
            pipeline_path: Path to pipeline code

        Returns:
            List of import-error strings (empty when everything resolves)
        """
        errors: List[str] = []

        for py_file in Path(pipeline_path).rglob("*.py"):
            try:
                source = py_file.read_text()
                tree = ast.parse(source, filename=str(py_file))
            except SyntaxError:
                continue
            except Exception as exc:
                errors.append(f"Could not read {py_file}: {exc}")
                continue

            for node in ast.walk(tree):
                if not isinstance(node, (ast.Import, ast.ImportFrom)):
                    continue

                if isinstance(node, ast.Import):
                    module_names = [alias.name.split(".")[0] for alias in node.names]
                else:
                    if node.level and node.level > 0:
                        continue
                    module_names = (
                        [node.module.split(".")[0]] if node.module else []
                    )

                for module_name in module_names:
                    if not module_name:
                        continue
                    try:
                        spec = importlib.util.find_spec(module_name)
                        if spec is None:
                            errors.append(
                                f"{py_file}: cannot resolve import '{module_name}'"
                            )
                    except (ModuleNotFoundError, ValueError):
                        errors.append(
                            f"{py_file}: cannot resolve import '{module_name}'"
                        )

        return errors

    def cleanup(self) -> None:
        """Clean up temporary files and backups."""
        if self.backup_dir and self.backup_dir.exists():
            shutil.rmtree(self.backup_dir)
            self.backup_dir = None
