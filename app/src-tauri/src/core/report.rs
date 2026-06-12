use serde::{Deserialize, Serialize};

use crate::core::deep_research::JournalCandidate;
use crate::core::summary::SummaryResult;
use docx_rs::{Docx, Paragraph, Run};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReportData {
    pub markdown: String,
    pub json: String,
    pub docx_base64: String,
}

pub fn generate_report(summary: &SummaryResult, journals: &[JournalCandidate]) -> ReportData {
    let markdown = generate_markdown(summary, journals);
    let json = generate_json(summary, journals);
    let docx_base64 = generate_docx_base64(summary, journals);
    ReportData { markdown, json, docx_base64 }
}

fn generate_markdown(summary: &SummaryResult, journals: &[JournalCandidate]) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("# 投稿先選定レポート — 論文投稿先アドバイザ".to_string());
    lines.push(String::new());

    // Section 1: Summary
    lines.push("---".to_string());
    lines.push(String::new());
    lines.push("## 1. 論文の構造化要約".to_string());
    lines.push(String::new());
    lines.push(format!("**研究テーマ**: {}", summary.research_topic));
    lines.push(String::new());
    lines.push(format!("**目的**: {}", summary.objective));
    lines.push(String::new());
    lines.push(format!("**対象**: {}", summary.sample_summary));
    lines.push(String::new());
    lines.push(format!("**研究デザイン**: {}", summary.design));
    lines.push(String::new());
    lines.push(format!("**方法**: {}", summary.methods_summary));
    lines.push(String::new());
    lines.push(format!("**測定指標**: {}", summary.measures));
    lines.push(String::new());
    lines.push(format!("**統計手法**: {}", summary.statistics));
    lines.push(String::new());
    lines.push(format!("**主要結果**: {}", summary.findings));
    lines.push(String::new());
    lines.push(format!("**著者の主張する貢献**: {}", summary.claimed_contributions));
    lines.push(String::new());

    // Section 2: Keywords
    lines.push("## 2. 検索キーワード".to_string());
    lines.push(String::new());
    lines.push(summary.keywords_for_search.join(", "));
    lines.push(String::new());

    // Section 3: Journal overview table
    lines.push("---".to_string());
    lines.push(String::new());
    lines.push(format!("## 3. 推薦ジャーナル一覧 ({} 件)", journals.len()));
    lines.push(String::new());
    lines.push("| # | ジャーナル名 | マッチ度 | IF/指標 | APC | 掲載方法 | 費用リスク | 推薦 |".to_string());
    lines.push("|---|---|---|---|---|---|---|---|".to_string());
    for (i, j) in journals.iter().enumerate() {
        lines.push(format!(
            "| {} | {} | {}% | {} | {} | {} | {} | {} |",
            i + 1, j.journal_name, j.match_score, j.impact_factor_or_metric,
            j.apc, j.publication_route, j.cost_risk_level, j.recommendation_level
        ));
    }
    lines.push(String::new());

    // Section 4: Journal details
    lines.push("## 4. ジャーナル詳細".to_string());
    lines.push(String::new());
    for (i, j) in journals.iter().enumerate() {
        lines.push(format!("### {}. {} (マッチ度: {}%)", i + 1, j.journal_name, j.match_score));
        lines.push(String::new());
        lines.push(format!("- **出版社**: {}", j.publisher));
        lines.push(format!("- **スコープ適合**: {}", j.scope_fit));
        lines.push(format!("- **記事タイプ適合**: {}", j.article_type_fit));
        lines.push(format!("- **類似論文**: {}", j.similar_articles));
        lines.push(format!("- **IF/指標**: {}", j.impact_factor_or_metric));
        lines.push(format!("- **掲載方法 / Publication Route**: {}", j.publication_route));
        lines.push(format!("- **APC**: {}", j.apc));
        lines.push(format!("- **APC 必須/任意**: {}", j.apc_required));
        lines.push(format!("- **APC 回避の可能性**: {}", j.apc_avoidance));
        lines.push(format!("- **waiver / 割引情報**: {}", j.waiver_or_discount_info));
        lines.push(format!("- **費用リスクレベル**: {}", j.cost_risk_level));
        lines.push(format!("- **推奨投稿方法**: {}", j.recommended_submission_strategy));
        lines.push(format!("- **文字数制限**: {}", j.word_limit));
        lines.push(format!("- **OA ポリシー**: {}", j.open_access_policy));
        lines.push(format!("- **メリット**: {}", j.pros));
        lines.push(format!("- **デメリット**: {}", j.cons));
        lines.push(format!("- **推薦レベル**: {}", j.recommendation_level));
        lines.push(format!("- **推薦理由**: {}", j.reason));
        lines.push(format!("- **根拠**: {}", j.source_evidence));
        lines.push(String::new());
    }

    // Section 5: Low-cost submission strategy
    lines.push("---".to_string());
    lines.push(String::new());
    lines.push("## 5. 低コスト投稿戦略".to_string());
    lines.push(String::new());

    let no_apc: Vec<&JournalCandidate> = journals.iter()
        .filter(|j| j.cost_risk_level == "low" || j.apc_required == "no_apc")
        .collect();
    let optional_apc: Vec<&JournalCandidate> = journals.iter()
        .filter(|j| j.apc_required == "optional")
        .collect();
    let required_apc: Vec<&JournalCandidate> = journals.iter()
        .filter(|j| j.apc_required == "required")
        .collect();
    let waiver_possible: Vec<&JournalCandidate> = journals.iter()
        .filter(|j| j.waiver_or_discount_info.to_lowercase().contains("waiver")
            || j.waiver_or_discount_info.contains("割引")
            || j.waiver_or_discount_info.contains("機関契約"))
        .collect();

    lines.push("### APC なしで投稿・掲載可能な候補".to_string());
    lines.push(String::new());
    if no_apc.is_empty() {
        lines.push("該当なし".to_string());
    } else {
        for j in &no_apc {
            lines.push(format!("- **{}** (マッチ度 {}%) — {}", j.journal_name, j.match_score, j.recommended_submission_strategy));
        }
    }
    lines.push(String::new());

    lines.push("### Hybrid 誌で非 OA 投稿が可能な候補".to_string());
    lines.push(String::new());
    if optional_apc.is_empty() {
        lines.push("該当なし".to_string());
    } else {
        for j in &optional_apc {
            lines.push(format!("- **{}** (マッチ度 {}%) — {}", j.journal_name, j.match_score, j.recommended_submission_strategy));
        }
    }
    lines.push(String::new());

    if !required_apc.is_empty() {
        lines.push("### APC 必須のため注意が必要な候補".to_string());
        lines.push(String::new());
        for j in &required_apc {
            lines.push(format!("- **{}** (マッチ度 {}%) — APC: {} — {}", j.journal_name, j.match_score, j.apc, j.recommended_submission_strategy));
        }
        lines.push(String::new());
    }

    if !waiver_possible.is_empty() {
        lines.push("### waiver / 割引確認が必要な候補".to_string());
        lines.push(String::new());
        for j in &waiver_possible {
            lines.push(format!("- **{}** — {}", j.journal_name, j.waiver_or_discount_info));
        }
        lines.push(String::new());
    }

    // First choice recommendation (low cost priority)
    lines.push("### 最初に投稿すべき安価な第一候補".to_string());
    lines.push(String::new());
    let best_low_cost = no_apc.iter()
        .chain(optional_apc.iter())
        .max_by_key(|j| j.match_score);
    match best_low_cost {
        Some(j) => lines.push(format!(
            "**{}** (マッチ度 {}%) — {}",
            j.journal_name, j.match_score, j.recommended_submission_strategy
        )),
        None => lines.push("安価な候補が見つかりませんでした。APC 必須の候補を検討してください。".to_string()),
    }
    lines.push(String::new());

    // High-cost challenge
    let high_match_with_apc: Vec<&&JournalCandidate> = required_apc.iter()
        .filter(|j| j.match_score >= 70)
        .collect::<Vec<_>>()
        .into_iter()
        .collect();
    if !high_match_with_apc.is_empty() {
        lines.push("### コストは高いがマッチ度が高いチャレンジ候補".to_string());
        lines.push(String::new());
        for j in &high_match_with_apc {
            lines.push(format!(
                "- **{}** (マッチ度 {}%) — APC: {} — {}",
                j.journal_name, j.match_score, j.apc, j.recommended_submission_strategy
            ));
        }
        lines.push(String::new());
    }

    // Section 6: Final recommendation
    lines.push("---".to_string());
    lines.push(String::new());
    lines.push("## 6. 最終推奨".to_string());
    lines.push(String::new());

    let strong: Vec<&JournalCandidate> = journals.iter()
        .filter(|j| j.recommendation_level.to_lowercase() == "strong")
        .collect();
    let moderate: Vec<&JournalCandidate> = journals.iter()
        .filter(|j| j.recommendation_level.to_lowercase() == "moderate")
        .collect();
    let weak: Vec<&JournalCandidate> = journals.iter()
        .filter(|j| j.recommendation_level.to_lowercase() == "weak")
        .collect();

    lines.push("### 第一候補（推薦レベル: Strong）".to_string());
    lines.push(String::new());
    if strong.is_empty() {
        lines.push("該当なし".to_string());
    } else {
        for j in &strong {
            lines.push(format!("- **{}** (マッチ度 {}%, 費用リスク: {}) — {}", j.journal_name, j.match_score, j.cost_risk_level, j.reason));
        }
    }
    lines.push(String::new());

    lines.push("### 第二候補（推薦レベル: Moderate）".to_string());
    lines.push(String::new());
    if moderate.is_empty() {
        lines.push("該当なし".to_string());
    } else {
        for j in &moderate {
            lines.push(format!("- **{}** (マッチ度 {}%, 費用リスク: {}) — {}", j.journal_name, j.match_score, j.cost_risk_level, j.reason));
        }
    }
    lines.push(String::new());

    if !weak.is_empty() {
        lines.push("### チャレンジ候補（推薦レベル: Weak）".to_string());
        lines.push(String::new());
        for j in &weak {
            lines.push(format!("- **{}** (マッチ度 {}%, 費用リスク: {}) — {}", j.journal_name, j.match_score, j.cost_risk_level, j.reason));
        }
        lines.push(String::new());
    }

    // Section 7: Disclaimers
    lines.push("---".to_string());
    lines.push(String::new());
    lines.push("## 7. 注意書き".to_string());
    lines.push(String::new());
    lines.push("- **Impact Factor・CiteScore・SJR などの指標は年ごとに変動します。投稿前に必ず各ジャーナルの公式サイトで最新値を確認してください。**".to_string());
    lines.push("- **APC（Article Processing Charge）の金額、必須/任意の条件、waiver の有無は変更される可能性があります。投稿前に各ジャーナルの公式サイトで確認してください。**".to_string());
    lines.push("- **掲載方法（Subscription / Hybrid OA / Gold OA）や非 OA 投稿の可否は、出版社のポリシー変更により変わる場合があります。**".to_string());
    lines.push("- **所属機関の Read & Publish 契約（Transformative Agreement）があるかどうかは、大学図書館または機関の研究支援部門に確認してください。**".to_string());
    lines.push("- **Impact Factor は Clarivate Journal Citation Reports (JCR) 由来の指標であり、外部 Deep Research では取得できない場合があります。Q1/Q2 や SJR は代替指標として表示しています。**".to_string());
    lines.push("- **本レポートの Deep Research 結果は AI による自動推定であり、査読結果や採択可否を保証するものではありません。**".to_string());
    lines.push("- **最終的な投稿先の選定は、研究者自身の判断で行ってください。**".to_string());
    lines.push(String::new());
    lines.push("---".to_string());
    lines.push("*Generated by 論文投稿先アドバイザ*".to_string());

    lines.join("\n")
}

fn generate_json(summary: &SummaryResult, journals: &[JournalCandidate]) -> String {
    let report = serde_json::json!({
        "summary": {
            "research_topic": summary.research_topic,
            "objective": summary.objective,
            "sample_summary": summary.sample_summary,
            "design": summary.design,
            "methods_summary": summary.methods_summary,
            "measures": summary.measures,
            "statistics": summary.statistics,
            "findings": summary.findings,
            "claimed_contributions": summary.claimed_contributions,
            "keywords_for_search": summary.keywords_for_search,
        },
        "journals": journals.iter().map(|j| serde_json::json!({
            "journal_name": j.journal_name,
            "publisher": j.publisher,
            "scope_fit": j.scope_fit,
            "article_type_fit": j.article_type_fit,
            "similar_articles": j.similar_articles,
            "impact_factor_or_metric": j.impact_factor_or_metric,
            "apc": j.apc,
            "word_limit": j.word_limit,
            "open_access_policy": j.open_access_policy,
            "pros": j.pros,
            "cons": j.cons,
            "recommendation_level": j.recommendation_level,
            "reason": j.reason,
            "source_evidence": j.source_evidence,
            "match_score": j.match_score,
            "publication_route": j.publication_route,
            "apc_required": j.apc_required,
            "apc_avoidance": j.apc_avoidance,
            "recommended_submission_strategy": j.recommended_submission_strategy,
            "waiver_or_discount_info": j.waiver_or_discount_info,
            "cost_risk_level": j.cost_risk_level,
        })).collect::<Vec<_>>(),
        "low_cost_strategy": {
            "no_apc_candidates": journals.iter()
                .filter(|j| j.cost_risk_level == "low" || j.apc_required == "no_apc")
                .map(|j| j.journal_name.clone())
                .collect::<Vec<_>>(),
            "optional_apc_candidates": journals.iter()
                .filter(|j| j.apc_required == "optional")
                .map(|j| j.journal_name.clone())
                .collect::<Vec<_>>(),
            "required_apc_candidates": journals.iter()
                .filter(|j| j.apc_required == "required")
                .map(|j| j.journal_name.clone())
                .collect::<Vec<_>>(),
            "waiver_possible_candidates": journals.iter()
                .filter(|j| j.waiver_or_discount_info.to_lowercase().contains("waiver")
                    || j.waiver_or_discount_info.contains("割引")
                    || j.waiver_or_discount_info.contains("機関契約"))
                .map(|j| j.journal_name.clone())
                .collect::<Vec<_>>(),
        },
        "recommendation": {
            "first_choice": journals.iter()
                .filter(|j| j.recommendation_level.to_lowercase() == "strong")
                .map(|j| serde_json::json!({"journal_name": j.journal_name, "match_score": j.match_score, "cost_risk_level": j.cost_risk_level}))
                .collect::<Vec<_>>(),
            "second_choice": journals.iter()
                .filter(|j| j.recommendation_level.to_lowercase() == "moderate")
                .map(|j| serde_json::json!({"journal_name": j.journal_name, "match_score": j.match_score, "cost_risk_level": j.cost_risk_level}))
                .collect::<Vec<_>>(),
            "challenge": journals.iter()
                .filter(|j| j.recommendation_level.to_lowercase() == "weak")
                .map(|j| serde_json::json!({"journal_name": j.journal_name, "match_score": j.match_score, "cost_risk_level": j.cost_risk_level}))
                .collect::<Vec<_>>(),
        },
        "disclaimer": "IF/APC/publication routes may change. Always verify on the official journal website before submission. This report is AI-generated and does not guarantee acceptance."
    });

    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}

fn generate_docx_base64(summary: &SummaryResult, journals: &[JournalCandidate]) -> String {
    use docx_rs::*;

    let mut doc = Docx::new();

    // Title
    doc = doc.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text("投稿先選定レポート — 論文投稿先アドバイザ").bold().size(28)),
    );
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("")));

    // Section 1: Summary
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("1. 論文の構造化要約").bold().size(24)));
    doc = add_field(doc, "研究テーマ", &summary.research_topic);
    doc = add_field(doc, "目的", &summary.objective);
    doc = add_field(doc, "対象", &summary.sample_summary);
    doc = add_field(doc, "研究デザイン", &summary.design);
    doc = add_field(doc, "方法", &summary.methods_summary);
    doc = add_field(doc, "主要結果", &summary.findings);

    // Section 2: Keywords
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("2. 検索キーワード").bold().size(24)));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&summary.keywords_for_search.join(", "))));

    // Section 3: Journal table
    doc = doc.add_paragraph(
        Paragraph::new().add_run(Run::new().add_text(&format!("3. 推薦ジャーナル一覧 ({} 件)", journals.len())).bold().size(24)),
    );

    let header_row = TableRow::new(vec![
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("#").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("ジャーナル名").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("マッチ度").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("指標").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("APC").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("費用リスク").bold())),
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text("推薦").bold())),
    ]);

    let mut rows = vec![header_row];
    for (i, j) in journals.iter().enumerate() {
        rows.push(TableRow::new(vec![
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{}", i + 1)))),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(&j.journal_name).bold())),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(&format!("{}%", j.match_score)))),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(&j.impact_factor_or_metric))),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(&j.apc))),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(&j.cost_risk_level))),
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(&j.recommendation_level))),
        ]));
    }
    doc = doc.add_table(Table::new(rows));

    // Section 4: Journal details
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("4. ジャーナル詳細").bold().size(24)));
    for (i, j) in journals.iter().enumerate() {
        doc = doc.add_paragraph(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(&format!("{}. {} (マッチ度: {}%)", i + 1, j.journal_name, j.match_score))
                    .bold()
                    .size(22),
            ),
        );
        doc = add_field(doc, "出版社", &j.publisher);
        doc = add_field(doc, "スコープ適合", &j.scope_fit);
        doc = add_field(doc, "掲載方法", &j.publication_route);
        doc = add_field(doc, "APC", &j.apc);
        doc = add_field(doc, "APC 必須/任意", &j.apc_required);
        doc = add_field(doc, "APC 回避", &j.apc_avoidance);
        doc = add_field(doc, "推奨投稿方法", &j.recommended_submission_strategy);
        doc = add_field(doc, "推薦理由", &j.reason);
    }

    // Section 5: Disclaimers
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("5. 注意書き").bold().size(24)));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("- Impact Factor は Clarivate JCR 由来の指標であり、外部 Deep Research では取得できない場合があります。")));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("- APC、掲載方法、waiver の有無は変更される可能性があります。")));
    doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text("- 本レポートは AI による自動推定であり、査読結果や採択可否を保証するものではありません。")));

    // Generate bytes and encode to base64
    let mut buf = std::io::Cursor::new(Vec::new());
    match doc.build().pack(&mut buf) {
        Ok(()) => {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.encode(buf.into_inner())
        }
        Err(_) => String::new(),
    }
}

fn add_field(doc: Docx, label: &str, value: &str) -> Docx {
    doc.add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text(&format!("{}: ", label)).bold())
            .add_run(Run::new().add_text(value)),
    )
}

