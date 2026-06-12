use crate::core::docx_extract::ManuscriptText;
use crate::core::summary::SummaryResult;

pub fn build_summary_messages(manuscript: &ManuscriptText) -> Vec<crate::core::llm_client::ChatMessage> {
    let system = r#"You are an expert assistant for academic manuscript analysis.
Read the manuscript provided and produce a structured summary.

CRITICAL OUTPUT RULES:
- Return ONLY valid JSON.
- Do NOT include Markdown fences (```json).
- Do NOT include any text outside the JSON object.
- Do NOT include explanations before or after the JSON.
- All values must be strings (use "Not stated" if information is absent).
- keywords_for_search must be a JSON array of strings.

Output this exact JSON structure:
{
  "research_topic": "Research topic in 1-2 sentences",
  "objective": "Research objective in 1-2 sentences",
  "sample_summary": "Summary of participants/sample including demographics, setting, sample size",
  "design": "Study design (e.g., RCT, cross-sectional, longitudinal, qualitative)",
  "methods_summary": "Summary of intervention/survey/observation methods",
  "measures": "List of measures/outcome indicators used",
  "statistics": "Statistical analysis methods used",
  "findings": "Main results (primary and secondary outcomes)",
  "claimed_contributions": "Contributions claimed by the authors",
  "keywords_for_search": ["keyword1", "keyword2", "keyword3", "keyword4", "keyword5"]
}"#;

    let mut user_parts = vec!["# Manuscript\n".to_string()];

    if !manuscript.sections.is_empty() {
        for sec in &manuscript.sections {
            user_parts.push(format!("## {}\n{}\n", sec.heading, sec.body));
        }
    } else {
        user_parts.push(manuscript.raw_text.clone());
    }

    let mut user_text = user_parts.join("\n");
    if user_text.len() > 100_000 {
        user_text.truncate(100_000);
        user_text.push_str("\n\n[Text truncated due to length]");
    }

    vec![
        crate::core::llm_client::ChatMessage {
            role: "system".to_string(),
            content: system.to_string(),
        },
        crate::core::llm_client::ChatMessage {
            role: "user".to_string(),
            content: user_text,
        },
    ]
}

pub fn build_deep_research_prompt(summary: &SummaryResult) -> String {
    let keywords = summary.keywords_for_search.join(", ");

    format!(
        r#"You are a research assistant specializing in academic journal selection.
Based on the manuscript summary below, find 5-10 real, existing academic journals that would be suitable submission targets.

## Manuscript Information
- Research Topic: {topic}
- Objective: {objective}
- Sample: {sample}
- Design: {design}
- Methods: {methods}
- Findings: {findings}
- Keywords: {keywords}

## Task
Search for real academic journals that match this manuscript. For each journal, provide ALL of the following:

1. **Journal Name** — full official name
2. **Publisher**
3. **Scope Fit** — how well the journal's scope matches this research topic
4. **Article Type Fit** — does this journal accept this type of article (original research, review, etc.)
5. **Similar Articles** — examples of similar articles published in this journal (if known)
6. **Impact Factor / Metric** — current IF, CiteScore, or other ranking metric
7. **APC** — article processing charge (e.g. "$2,000", "Free", "€1,500")
8. **Word Limit** — manuscript word limit if any
9. **Open Access Policy** — gold OA, hybrid, green, or subscription
10. **Pros** — advantages of submitting to this journal
11. **Cons** — disadvantages or risks
12. **Recommendation Level** — Strong / Moderate / Weak
13. **Reason** — concise reason why this journal fits (1-2 sentences)
14. **Source Evidence** — what evidence supports this recommendation (URL, prior knowledge, etc.)

Use your web search capability if available. Provide current, accurate information.
Rank by fit quality (best fit first)."#,
        topic = summary.research_topic,
        objective = summary.objective,
        sample = summary.sample_summary,
        design = summary.design,
        methods = summary.methods_summary,
        findings = summary.findings,
        keywords = keywords,
    )
}

pub fn build_parse_external_messages(
    summary: &SummaryResult,
    external_a: &str,
    external_b: Option<&str>,
) -> Vec<crate::core::llm_client::ChatMessage> {
    let has_b = external_b.is_some() && !external_b.unwrap().trim().is_empty();

    let merge_instruction = if has_b {
        "Read both external AI results. Merge the journal candidates: identify duplicates (same journal in both A and B), keep the more detailed metadata, and combine reasons. Exclude clearly unsuitable journals. Rank by fit quality (best fit first). Include ALL candidates found — do NOT limit to top 10."
    } else {
        "Read the external AI results. Extract ALL journal candidates. Exclude clearly unsuitable journals. Rank by fit quality (best fit first). Include ALL candidates found — do NOT limit to top 10."
    };

    let system = format!(
        r#"You are a coordinating peer reviewer. Parse and merge academic journal recommendations from external AI search results into a clean, ranked list.

CRITICAL OUTPUT RULES:
- Return ONLY valid JSON.
- Do NOT include Markdown fences (```json).
- Do NOT include any text outside the JSON array.
- All values must be strings (use "" if information is not available).
- match_score must be an integer 0-100 (as a string, e.g. "85").
- Journal names and publisher names must be in English.
- Include ALL candidates — do NOT limit the number.

{merge_instruction}

Output a JSON array of objects. Each object must have these exact fields:
[
  {{
    "journal_name": "Full official journal name",
    "publisher": "Publisher name",
    "scope_fit": "How well the journal scope matches this research",
    "article_type_fit": "Does this journal accept this article type",
    "similar_articles": "Examples of similar articles in this journal",
    "impact_factor_or_metric": "IF, CiteScore, or other metric",
    "apc": "Article processing charge amount (e.g. '$2,000', 'Free', '€1,500')",
    "word_limit": "Manuscript word limit",
    "open_access_policy": "Gold OA / Hybrid / Green / Subscription",
    "pros": "Advantages of submitting here",
    "cons": "Disadvantages or risks",
    "recommendation_level": "Strong / Moderate / Weak",
    "reason": "1-2 sentence reason why this journal fits",
    "source_evidence": "Evidence supporting this recommendation",
    "match_score": "0-100 integer score representing how well the manuscript fits this journal",
    "publication_route": "Possible publication routes: Subscription/traditional, Hybrid OA, Gold OA, Green OA/self-archiving, etc.",
    "apc_required": "APC status: 'required' (APC mandatory), 'optional' (APC optional for OA), 'no_apc' (no APC needed), or 'unknown'",
    "apc_avoidance": "Can APC be avoided? E.g. 'APCなしで投稿・掲載可能', 'Hybrid誌だが非OA選択可', 'Gold OAのためAPC必須', 'waiver可能性あり', '要確認'",
    "recommended_submission_strategy": "Cost-optimal strategy in Japanese. E.g. 'Open Accessを選ばず、通常投稿として出す', 'APC必須のため研究費がない場合は優先度を下げる', etc.",
    "waiver_or_discount_info": "Waiver/discount availability. E.g. 'APC waiver対象の可能性あり', '機関契約あり', '低所得国waiverあり', '情報なし'",
    "cost_risk_level": "Cost risk: 'low' (free or avoidable), 'medium' (optional APC or waiver possible), 'high' (mandatory APC), 'unknown'"
  }}
]"#
    );

    let keywords = summary.keywords_for_search.join(", ");
    let mut user_parts = vec![
        format!(
            "# Manuscript Summary\n- Topic: {}\n- Objective: {}\n- Sample: {}\n- Design: {}\n- Findings: {}\n- Keywords: {}",
            summary.research_topic, summary.objective, summary.sample_summary,
            summary.design, summary.findings, keywords
        ),
        format!("\n# External AI A — Search Results\n{}", external_a),
    ];

    if let Some(b) = external_b {
        if !b.trim().is_empty() {
            user_parts.push(format!("\n# External AI B — Search Results\n{}", b));
        }
    }

    vec![
        crate::core::llm_client::ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        crate::core::llm_client::ChatMessage {
            role: "user".to_string(),
            content: user_parts.join("\n\n"),
        },
    ]
}
