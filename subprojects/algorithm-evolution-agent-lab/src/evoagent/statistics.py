from __future__ import annotations

import math
from dataclasses import dataclass


@dataclass(frozen=True)
class ConfidenceInterval:
    mean: float
    lower: float
    upper: float
    confidence_level: float
    n: int


# cf-atom: CODE-ConfidenceIntervalEstimator
class ConfidenceIntervalEstimator:
    def estimate(self, values: tuple[float, ...] | list[float], *, confidence_level: float = 0.95) -> ConfidenceInterval:
        if not values:
            raise ValueError("at least one value is required")
        if confidence_level != 0.95:
            raise ValueError("only 0.95 confidence_level is currently supported")
        numeric_values = tuple(float(value) for value in values)
        mean = sum(numeric_values) / len(numeric_values)
        if len(numeric_values) == 1:
            return ConfidenceInterval(mean=mean, lower=mean, upper=mean, confidence_level=confidence_level, n=1)

        variance = sum((value - mean) ** 2 for value in numeric_values) / (len(numeric_values) - 1)
        standard_error = math.sqrt(variance) / math.sqrt(len(numeric_values))
        margin = 1.96 * standard_error
        return ConfidenceInterval(
            mean=mean,
            lower=mean - margin,
            upper=mean + margin,
            confidence_level=confidence_level,
            n=len(numeric_values),
        )
