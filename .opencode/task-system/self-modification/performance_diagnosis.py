"""
Pipeline diagnosis module for analyzing campaign executor performance.

This module provides functionality to diagnose performance issues in the campaign
executor's pipeline execution, analyze task health, and generate improvement suggestions.
"""

from typing import Dict, List, Any, Optional
from dataclasses import dataclass, field
from pathlib import Path
import json
import asyncio


@dataclass
class PipelineReport:
    """Report containing pipeline diagnosis results."""
    overall_score: float
    benchmark_scores: Dict[str, float]
    improvement_suggestions: List[str] = field(default_factory=list)
    pipeline_health_issues: List[str] = field(default_factory=list)
    budget_issues: List[str] = field(default_factory=list)
    stagnation_patterns: List[str] = field(default_factory=list)
    error_rate_issues: List[str] = field(default_factory=list)
    recitation_issues: List[str] = field(default_factory=list)
    high_priority_areas: List[str] = field(default_factory=list)
    detailed_results: Optional[Dict[str, Any]] = None


class PipelineDiagnosis:
    """
    Analyzes pipeline performance and identifies areas for improvement.

    This class provides methods to diagnose performance issues by analyzing
    task completion rates, budget usage, stagnation patterns, and error rates.
    """

    def __init__(self):
        """Initialize the pipeline diagnoser."""
        self.min_acceptable_score = 0.7
        self.critical_score_threshold = 0.5

    async def diagnose_performance(
        self,
        pipeline_path: str,
        benchmark_results: Dict[str, Any]
    ) -> PipelineReport:
        """
        Perform comprehensive pipeline diagnosis.

        Args:
            pipeline_path: Path to the pipeline code
            benchmark_results: Results from benchmark evaluation

        Returns:
            PipelineReport: Comprehensive diagnosis report
        """
        report = PipelineReport(
            overall_score=benchmark_results.get('overall_score', 0.0),
            benchmark_scores=benchmark_results.get('benchmark_scores', {}),
            detailed_results=benchmark_results.get('detailed_results', {})
        )

        # Analyze different aspects
        await self._analyze_pipeline_health(pipeline_path, report)
        self._analyze_budget_usage(pipeline_path, report)
        self._analyze_benchmark_failures(benchmark_results, report)
        self._generate_improvement_suggestions(report)

        return report

    async def _analyze_pipeline_health(
        self,
        pipeline_path: str,
        report: PipelineReport
    ) -> None:
        """
        Analyze pipeline health for potential issues.

        Args:
            pipeline_path: Path to pipeline code
            report: Report to update with findings
        """
        path = Path(pipeline_path)

        # Analyze main campaign file
        plan_dir = path / "docs" / "plans"
        if plan_dir.exists():
            plan_files = list(plan_dir.glob("*.md"))
            if not plan_files:
                report.pipeline_health_issues.append(
                    "No plan files found - pipeline has no active tasks"
                )
            else:
                for plan_file in plan_files:
                    content = plan_file.read_text()

                    # Check recitation completeness
                    if "recitation" not in content.lower():
                        report.recitation_issues.append(
                            f"{plan_file.name}: No recitation tracking found"
                        )

                    # Check for stalled patterns
                    if "stalled" in content.lower() or "blocked" in content.lower():
                        report.stagnation_patterns.append(
                            f"{plan_file.name}: Contains stalled or blocked tasks"
                        )

        # Check campaign logs for error patterns
        logs_dir = path / ".campaign" / "logs"
        if logs_dir.exists():
            log_errors = 0
            log_total = 0
            for log_file in logs_dir.glob("*.log"):
                log_total += 1
                content = log_file.read_text()
                if "ERROR" in content or "FAILED" in content:
                    log_errors += 1

            if log_total > 0:
                error_rate = log_errors / log_total
                if error_rate > 0.3:
                    report.error_rate_issues.append(
                        f"High error rate: {error_rate:.0%} of log files contain errors"
                    )

        # Check for agent artifacts
        agent_state_file = path / ".campaign" / "agent_state.json"
        if agent_state_file.exists():
            try:
                state = json.loads(agent_state_file.read_text())
                completed = state.get("tasks_completed", 0)
                total = state.get("tasks_total", 0)
                if total > 0 and completed / total < 0.5:
                    report.pipeline_health_issues.append(
                        f"Low task completion rate: {completed}/{total} tasks completed"
                    )
            except (json.JSONDecodeError, KeyError):
                report.pipeline_health_issues.append(
                    "Corrupt agent state file - cannot read task metrics"
                )

    def _analyze_budget_usage(self, pipeline_path: str, report: PipelineReport) -> None:
        """
        Analyze budget usage patterns.

        Args:
            pipeline_path: Path to pipeline code
            report: Report to update with findings
        """
        path = Path(pipeline_path)

        budget_file = path / ".campaign" / "budget.json"
        if budget_file.exists():
            try:
                budget = json.loads(budget_file.read_text())
                used = budget.get("budget_used", 0)
                limit = budget.get("budget_limit", 0)
                if limit > 0 and used / limit > 0.8:
                    report.budget_issues.append(
                        f"Budget nearly exhausted: {used:.0f}/{limit:.0f} used ({used/limit:.0%})"
                    )
            except (json.JSONDecodeError, KeyError):
                report.budget_issues.append(
                    "Cannot parse budget file - possible corruption"
                )
        else:
            report.budget_issues.append(
                "No budget tracking found - pipeline may exceed limits undetected"
            )

    def _analyze_benchmark_failures(
        self,
        benchmark_results: Dict[str, Any],
        report: PipelineReport
    ) -> None:
        """
        Analyze patterns in benchmark failures.

        Args:
            benchmark_results: Results from benchmarks
            report: Report to update with findings
        """
        detailed_results = benchmark_results.get('detailed_results', {})

        for benchmark_name, results in detailed_results.items():
            if 'test_results' in results:
                timeout_count = 0
                error_types = {}

                for test in results['test_results']:
                    if not test.get('passed', True):
                        error = test.get('error', 'Unknown error')
                        if 'Timeout' in error:
                            timeout_count += 1
                        error_types[error] = error_types.get(error, 0) + 1

                if timeout_count > len(results['test_results']) * 0.3:
                    report.stagnation_patterns.append(
                        f"{benchmark_name}: {timeout_count} timeouts detected"
                    )

                for error, count in error_types.items():
                    if count > 1:
                        report.error_rate_issues.append(
                            f"{benchmark_name}: Repeated error - {error} ({count} times)"
                        )

    def _generate_improvement_suggestions(self, report: PipelineReport) -> None:
        """
        Generate improvement suggestions based on diagnosis.

        Args:
            report: Report to update with suggestions
        """
        # Critical performance issues
        if report.overall_score < self.critical_score_threshold:
            report.improvement_suggestions.append(
                "Critical: Overall performance below 50% - pipeline restructuring needed"
            )
            report.high_priority_areas.append("Pipeline Architecture")

        # Budget issues
        if report.budget_issues:
            report.improvement_suggestions.append(
                "Add budget limits and monitoring for better resource management"
            )
            report.high_priority_areas.append("Budget Management")

        # Stagnation patterns
        if report.stagnation_patterns:
            report.improvement_suggestions.append(
                "Detect and handle stalls - add retry logic with backoff"
            )
            report.high_priority_areas.append("Stall Recovery")

        # Error rate issues
        if report.error_rate_issues:
            report.improvement_suggestions.append(
                "Add comprehensive error handling and recovery mechanisms"
            )
            report.high_priority_areas.append("Error Handling")

        # Recitation issues
        if report.recitation_issues:
            report.improvement_suggestions.append(
                "Implement recitation tracking for better goal adherence"
            )

        # Pipeline health
        if report.pipeline_health_issues:
            report.improvement_suggestions.append(
                "Refactor pipeline structure - improve task completion rate"
            )

        # Benchmark-specific suggestions
        for benchmark, score in report.benchmark_scores.items():
            if score < self.min_acceptable_score:
                report.improvement_suggestions.append(
                    f"Focus on improving {benchmark} performance (current: {score:.2f})"
                )
