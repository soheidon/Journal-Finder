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
    // This is the combined prompt for backward compatibility
    let positioning = build_positioning_prompt(summary);
    let journal = build_journal_search_prompt(summary, "");
    format!("{}\n\n---\n\n{}", positioning, journal)
}

pub fn build_positioning_prompt(summary: &SummaryResult) -> String {
    let keywords = summary.keywords_for_search.join(", ");

    format!(
        r#"You are a research assistant specializing in academic literature surveys.
Based on the manuscript summary below, investigate where this paper stands in the existing literature.

## Manuscript Information
- Research Topic: {topic}
- Objective: {objective}
- Sample: {sample}
- Design: {design}
- Methods: {methods}
- Findings: {findings}
- Keywords: {keywords}

## Task
Investigate the following and provide a detailed positioning report:

1. **Closest Prior Studies**
   - Find 5-10 representative papers on the same or very similar research topics.
   - For each: authors, year, title, journal, objective, sample, design, main findings.

2. **Novelty Assessment**
   - What is genuinely new about this manuscript compared to existing research?
   - Which aspects are already well-established?
   - Is this a replication, incremental extension, or significant contribution?

3. **Methodological Positioning**
   - How do the methods compare to prior work?
   - Are the methods standard, innovative, or unusual for this field?

4. **Research Domain**
   - Which research field(s) does this paper belong to?
   - Which sub-disciplines or specialties?
   - What journals typically publish this type of research?

5. **Keywords for Journal Search**
   - Propose 10-20 search keywords useful for finding suitable journals.
   - Include field-specific terms, methodology terms, population terms.

6. **Summary**
   - In 2-3 sentences, describe where this paper sits in the existing literature.
   - What type of journal would be the best fit (scope, tier, audience)?

Provide specific references with author names, year, title, journal name, and URL where available.
If evidence is uncertain, clearly state that it is uncertain."#,
        topic = summary.research_topic,
        objective = summary.objective,
        sample = summary.sample_summary,
        design = summary.design,
        methods = summary.methods_summary,
        findings = summary.findings,
        keywords = keywords,
    )
}

pub fn build_journal_search_prompt(summary: &SummaryResult, positioning: &str) -> String {
    let keywords = summary.keywords_for_search.join(", ");

    let positioning_section = if positioning.is_empty() {
        "(Positioning report not available. Proceed based on manuscript summary alone.)".to_string()
    } else {
        positioning.to_string()
    };

    format!(
        r#"You are a research assistant specializing in academic journal selection.
Based on the manuscript summary and the positioning report below, find suitable academic journals for submission.

## Manuscript Information
- Research Topic: {topic}
- Objective: {objective}
- Sample: {sample}
- Design: {design}
- Methods: {methods}
- Findings: {findings}
- Keywords: {keywords}

## Positioning Report (from prior research)
{positioning}

## Task
Search for real academic journals that match this manuscript. Find as many candidates as possible (do NOT limit to 10).
For each journal, provide ALL of the following:

1. **Journal Name** — full official name
2. **Publisher**
3. **Scope Fit** — how well the journal's scope matches (0-100%)
4. **Article Type Fit** — does this journal accept this type of article
5. **Similar Articles** — examples of similar articles published in this journal
6. **Impact Factor / Metric** — current IF, CiteScore, or other ranking
7. **APC** — article processing charge amount
8. **APC Required** — "required" / "optional" / "no_apc" / "unknown"
9. **APC Avoidance** — can APC be avoided? (e.g. non-OA option in hybrid journal)
10. **Waiver / Discount** — any waiver, discount, or Read & Publish possibilities
11. **Word Limit** — manuscript word limit
12. **Open Access Policy** — gold OA, hybrid, green, subscription
13. **Pros** — advantages
14. **Cons** — disadvantages
15. **Recommendation Level** — Strong / Moderate / Weak
16. **Reason** — why this journal fits
17. **Source Evidence** — URL or evidence
18. **Match Score** — 0-100 integer
19. **Publication Route** — subscription, hybrid OA, gold OA, etc.
20. **Recommended Submission Strategy** — cost-optimal strategy
21. **Cost Risk Level** — low / medium / high / unknown

Prioritize journals where APC can be avoided (subscription journals, hybrid with non-OA option).
Include all candidates found — do NOT limit the number."#,
        topic = summary.research_topic,
        objective = summary.objective,
        sample = summary.sample_summary,
        design = summary.design,
        methods = summary.methods_summary,
        findings = summary.findings,
        keywords = keywords,
        positioning = positioning_section,
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

pub fn build_deep_research_api_prompt(
    summary: &SummaryResult,
    positioning_report: &str,
) -> String {
    let keywords = summary.keywords_for_search.join(", ");
    let positioning_section = if positioning_report.is_empty() {
        "(Positioning report not available.)".to_string()
    } else {
        positioning_report.to_string()
    };

    format!(
        r#"You are a research assistant specializing in academic journal selection.
Find suitable submission journals for the manuscript below.

## Manuscript Summary
- Research Topic: {topic}
- Objective: {objective}
- Sample: {sample}
- Design: {design}
- Methods: {methods}
- Findings: {findings}
- Keywords: {keywords}

## Prior Research Positioning
{positioning}

## Task
Search for academic journals that would be good submission targets. For each journal, investigate:

1. Journal name and publisher
2. Scope fit with this manuscript (0-100%)
3. Whether similar articles have been published there
4. Article type compatibility
5. Impact factor or other ranking metric
6. APC — amount and whether required or optional
7. Whether APC can be avoided (e.g., non-OA option in hybrid journal)
8. Waiver / discount possibilities
9. Word limit, figure/table limits
10. Open access policy
11. Pros and cons
12. A low-cost submission strategy for each journal

Prioritize journals where APC can be avoided.
Find as many candidates as possible.
Provide URLs or evidence for your recommendations."#,
        topic = summary.research_topic,
        objective = summary.objective,
        sample = summary.sample_summary,
        design = summary.design,
        methods = summary.methods_summary,
        findings = summary.findings,
        keywords = keywords,
        positioning = positioning_section,
    )
}
