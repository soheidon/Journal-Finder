export function translateRecommendationLevel(level: string): string {
  switch (level.toLowerCase()) {
    case "strong": return "強く推奨";
    case "moderate": return "中程度に推奨";
    case "weak": return "優先度低め";
    default: return level;
  }
}

export function translateCostRisk(risk: string): string {
  switch (risk.toLowerCase()) {
    case "low": return "低";
    case "medium": return "中";
    case "high": return "高";
    case "unknown": return "不明";
    default: return risk;
  }
}

export function translateApcRequired(status: string): string {
  switch (status.toLowerCase()) {
    case "required": return "必須";
    case "optional": return "任意";
    case "no_apc": return "不要";
    case "unknown": return "不明";
    default: return status;
  }
}

export function translateCostRiskClass(risk: string): string {
  switch (risk.toLowerCase()) {
    case "low": return "low";
    case "medium": return "medium";
    case "high": return "high";
    default: return "unknown";
  }
}

export function formatMetricDisplay(j: {
  impact_factor_or_metric: string;
  quartile_or_rank: string;
  metric_source: string;
}): { label: string; value: string }[] {
  const result: { label: string; value: string }[] = [];

  const hasIf = j.impact_factor_or_metric && j.impact_factor_or_metric !== "未取得" && j.impact_factor_or_metric.toLowerCase() !== "n/a";
  const hasQuartile = j.quartile_or_rank && j.quartile_or_rank.trim().length > 0;

  if (hasIf) {
    result.push({ label: "Impact Factor", value: j.impact_factor_or_metric });
  } else {
    result.push({ label: "Impact Factor", value: "未取得" });
  }

  if (hasQuartile) {
    result.push({ label: "分野ランク", value: j.quartile_or_rank });
  }

  if (j.metric_source && j.metric_source !== "出典未確認") {
    result.push({ label: "指標ソース", value: j.metric_source });
  }

  return result;
}

