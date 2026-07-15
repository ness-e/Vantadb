"""
Modification proposal module for generating pipeline improvement proposals.

This module provides functionality to generate proposals for modifying the pipeline
configuration and behavior based on performance diagnosis results.
"""

from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field
from pathlib import Path
import uuid
import asyncio


@dataclass
class PipelineChange:
    """Represents a specific pipeline change to be made."""
    file_path: str
    change_type: str  # 'add', 'modify', 'delete'
    description: str
    priority: int
    location: Optional[str] = None  # e.g., 'config', 'retry_settings', etc.
    old_code: Optional[str] = None
    new_code: Optional[str] = None
    line_number: Optional[int] = None

    def __str__(self) -> str:
        return f"{self.change_type.upper()}: {self.description} (Priority: {self.priority})"


@dataclass
class PipelineProposal:
    """Proposal for modifying pipeline config to improve performance."""
    proposal_id: str
    diagnosis_summary: str
    code_changes: List[PipelineChange] = field(default_factory=list)
    implementation_steps: List[str] = field(default_factory=list)
    expected_improvements: List[str] = field(default_factory=list)
    risk_assessment: Optional[str] = None
    estimated_complexity: Optional[str] = None  # 'low', 'medium', 'high'

    def __str__(self) -> str:
        return f"Proposal {self.proposal_id}: {len(self.code_changes)} changes"


class PipelineModifier:
    """
    Generates modification proposals based on pipeline diagnosis.

    This class analyzes diagnosis reports and creates concrete proposals
    for pipeline modifications to improve campaign execution performance.
    """

    def __init__(self):
        """Initialize the pipeline modifier."""
        self.max_changes_per_proposal = 5
        self.improvement_strategies = {
            'pipeline_fix': self._propose_pipeline_fix,
            'error_handling': self._propose_error_handling,
            'stall_recovery': self._propose_stall_recovery,
            'budget_management': self._propose_budget_management
        }

    async def generate_proposal(
        self,
        diagnosis: 'PipelineReport',
        pipeline_path: str,
        target_improvements: Optional[List[str]] = None
    ) -> PipelineProposal:
        """
        Generate a modification proposal based on diagnosis.

        Args:
            diagnosis: Pipeline diagnosis report
            pipeline_path: Path to pipeline code
            target_improvements: Specific improvements to target

        Returns:
            PipelineProposal: Concrete proposal for modifications
        """
        proposal = PipelineProposal(
            proposal_id=str(uuid.uuid4())[:8],
            diagnosis_summary=self._summarize_diagnosis(diagnosis)
        )

        # Determine which improvements to prioritize
        priorities = self._prioritize_improvements(diagnosis, target_improvements)

        # Generate code changes for each priority area
        for priority_num, improvement_type in enumerate(priorities[:self.max_changes_per_proposal], 1):
            if improvement_type in self.improvement_strategies:
                changes = await self._generate_code_changes(
                    improvement_type,
                    diagnosis,
                    pipeline_path,
                    priority_num
                )
                proposal.code_changes.extend(changes)

        # Generate implementation steps
        proposal.implementation_steps = self._generate_implementation_steps(proposal)

        # Estimate improvements
        proposal.expected_improvements = self._estimate_improvements(proposal, diagnosis)

        # Assess risk and complexity
        proposal.risk_assessment = self._assess_risk(proposal)
        proposal.estimated_complexity = self._estimate_complexity(proposal)

        return proposal

    def _summarize_diagnosis(self, diagnosis: 'PipelineReport') -> str:
        """Create a summary of the diagnosis."""
        issues = []
        if diagnosis.budget_issues:
            issues.append(f"{len(diagnosis.budget_issues)} budget issues")
        if diagnosis.error_rate_issues:
            issues.append(f"{len(diagnosis.error_rate_issues)} error rate issues")
        if diagnosis.stagnation_patterns:
            issues.append(f"{len(diagnosis.stagnation_patterns)} stagnation patterns")

        return f"Pipeline performance score: {diagnosis.overall_score:.2f}. Issues: {', '.join(issues)}"

    def _prioritize_improvements(
        self,
        diagnosis: 'PipelineReport',
        target_improvements: Optional[List[str]] = None
    ) -> List[str]:
        """
        Prioritize which improvements to implement.

        Args:
            diagnosis: Pipeline diagnosis
            target_improvements: Specific requested improvements

        Returns:
            List of improvement types in priority order
        """
        priorities = []

        # Add targeted improvements first
        if target_improvements:
            priorities.extend(target_improvements)

        # Add critical improvements based on diagnosis
        if diagnosis.overall_score <= 0.5:
            if diagnosis.budget_issues:
                priorities.append('budget_management')
            if diagnosis.error_rate_issues:
                priorities.append('error_handling')

        # Add stall recovery if stagnation detected
        if diagnosis.stagnation_patterns:
            priorities.append('stall_recovery')

        # Add pipeline health improvements
        if diagnosis.pipeline_health_issues:
            priorities.append('pipeline_fix')

        # Remove duplicates while preserving order
        seen = set()
        unique_priorities = []
        for item in priorities:
            if item not in seen:
                seen.add(item)
                unique_priorities.append(item)

        return unique_priorities[:4]  # Limit to top 4 priorities

    async def _generate_code_changes(
        self,
        improvement_type: str,
        diagnosis: 'PipelineReport',
        pipeline_path: str,
        priority: int
    ) -> List[PipelineChange]:
        """
        Generate specific code changes for an improvement type.

        Args:
            improvement_type: Type of improvement
            diagnosis: Pipeline diagnosis
            pipeline_path: Path to pipeline code
            priority: Priority level for changes

        Returns:
            List of pipeline changes
        """
        strategy_func = self.improvement_strategies.get(improvement_type)
        if strategy_func:
            return await strategy_func(diagnosis, pipeline_path, priority)
        return []

    async def _propose_pipeline_fix(
        self,
        diagnosis: 'PipelineReport',
        pipeline_path: str,
        priority: int
    ) -> List[PipelineChange]:
        """Propose changes for pipeline health fixes."""
        changes = []

        # Add validation step config
        changes.append(PipelineChange(
            file_path='config/pipeline.yaml',
            change_type='add',
            location='validation',
            new_code='''validation:
  enabled: true
  pre_execution_checks:
    - recitation_check
    - budget_check
  post_execution_checks:
    - task_completion_verification
    - error_log_scan''',
            description='Add validation steps to pipeline config',
            priority=priority
        ))

        return changes

    async def _propose_error_handling(
        self,
        diagnosis: 'PipelineReport',
        pipeline_path: str,
        priority: int
    ) -> List[PipelineChange]:
        """Propose changes for error handling."""
        changes = []

        # Add retry config with limits
        changes.append(PipelineChange(
            file_path='config/pipeline.yaml',
            change_type='modify',
            location='retry_settings',
            description='Add retry count limits and backoff strategy',
            priority=priority
        ))

        return changes

    async def _propose_stall_recovery(
        self,
        diagnosis: 'PipelineReport',
        pipeline_path: str,
        priority: int
    ) -> List[PipelineChange]:
        """Propose changes for stall recovery."""
        changes = []

        # Add stall detection config
        changes.append(PipelineChange(
            file_path='config/pipeline.yaml',
            change_type='add',
            location='stall_detection',
            new_code='''stall_detection:
  enabled: true
  max_idle_iterations: 3
  recovery_strategy: auto_restart
  notification_on_stall: true''',
            description='Add stall detection and auto-recovery configuration',
            priority=priority
        ))

        # Add retry count adjustment
        changes.append(PipelineChange(
            file_path='config/pipeline.yaml',
            change_type='add',
            location='retry_policy',
            new_code='''retry_policy:
  max_retries: 3
  backoff_base_seconds: 5
  backoff_max_seconds: 120''',
            description='Add retry count limits with exponential backoff',
            priority=priority
        ))

        return changes

    async def _propose_budget_management(
        self,
        diagnosis: 'PipelineReport',
        pipeline_path: str,
        priority: int
    ) -> List[PipelineChange]:
        """Propose changes for budget management."""
        changes = []

        # Add budget limits config
        changes.append(PipelineChange(
            file_path='config/pipeline.yaml',
            change_type='add',
            location='budget_limits',
            new_code='''budget_limits:
  max_cost_per_task: 1000
  max_tasks_per_session: 50
  hard_stop_on_exceeded: true
  warning_threshold: 0.8''',
            description='Add budget limits and hard stop threshold',
            priority=priority
        ))

        return changes

    def _generate_implementation_steps(self, proposal: PipelineProposal) -> List[str]:
        """Generate step-by-step implementation instructions."""
        steps = []

        # Group changes by file
        changes_by_file = {}
        for change in proposal.code_changes:
            if change.file_path not in changes_by_file:
                changes_by_file[change.file_path] = []
            changes_by_file[change.file_path].append(change)

        # Generate steps for each file
        for file_path, changes in changes_by_file.items():
            steps.append(f"Modify {file_path}:")
            for change in sorted(changes, key=lambda x: x.priority):
                steps.append(f"  - {change.description}")

        steps.append("Run pipeline benchmarks to verify changes")
        steps.append("Validate improvements with campaign tests")

        return steps

    def _estimate_improvements(
        self,
        proposal: PipelineProposal,
        diagnosis: 'PipelineReport'
    ) -> List[str]:
        """Estimate expected improvements from the proposal."""
        improvements = []

        # Check for pipeline fix changes
        if any('validation' in change.description.lower() for change in proposal.code_changes):
            improvements.append("Improved task completion rate through validation")
            improvements.append(f"Expected score improvement: +{0.2:.1f} points")

        # Check for error handling
        if any('retry' in change.description.lower() or 'error' in change.description.lower()
               for change in proposal.code_changes):
            improvements.append("Improved reliability and error recovery")
            improvements.append("Reduced failure rate in benchmarks")

        # Check for stall recovery
        if any('stall' in change.description.lower() or 'retry' in change.description.lower()
               for change in proposal.code_changes):
            improvements.append("Faster recovery from stalled tasks")
            improvements.append("Reduced stagnation occurrences")

        # Check for budget management
        if any('budget' in change.description.lower() for change in proposal.code_changes):
            improvements.append("Controlled resource usage within limits")
            improvements.append("Early warning before budget exhaustion")

        return improvements

    def _assess_risk(self, proposal: PipelineProposal) -> str:
        """Assess the risk level of the proposed changes."""
        # Count high-impact changes
        high_impact = sum(1 for change in proposal.code_changes
                          if change.change_type in ['delete', 'modify'])

        if high_impact > 3:
            return "High risk - multiple core modifications"
        elif high_impact > 1:
            return "Medium risk - some core modifications"
        else:
            return "Low risk - mostly additions"

    def _estimate_complexity(self, proposal: PipelineProposal) -> str:
        """Estimate implementation complexity."""
        total_changes = len(proposal.code_changes)
        modify_changes = sum(1 for c in proposal.code_changes if c.change_type == 'modify')

        if total_changes > 4 or modify_changes > 2:
            return "high"
        elif total_changes > 2 or modify_changes > 0:
            return "medium"
        else:
            return "low"
