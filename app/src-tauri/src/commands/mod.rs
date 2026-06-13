pub mod docx;
pub mod journal;
pub mod llm;
pub mod project;
pub mod report;
pub mod summary;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ManuscriptText {
    pub raw_text: String,
    pub sections: Vec<Section>,
    pub paragraph_count: usize,
    pub char_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Section {
    pub heading: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SummaryResult {
    pub research_topic: String,
    pub objective: String,
    pub sample_summary: String,
    pub design: String,
    pub methods_summary: String,
    pub measures: String,
    pub statistics: String,
    pub findings: String,
    pub claimed_contributions: String,
    pub keywords_for_search: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JournalCandidate {
    pub journal_name: String,
    pub publisher: String,
    pub impact_factor: String,
    pub submission_fee: String,
    pub match_rate: String,
    pub reason: String,
}

#[tauri::command]
pub fn healthcheck() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": "ok",
        "app": "Journal Finder 論文投稿先アドバイザ",
        "version": "0.1.2",
        "rust_version": env!("CARGO_PKG_RUST_VERSION", "unknown"),
    }))
}

#[tauri::command]
pub fn get_dummy_manuscript() -> Result<ManuscriptText, String> {
    Ok(ManuscriptText {
        raw_text: "This is a sample manuscript for testing the Journal Finder UI.\n\n\
                   The purpose of this study was to examine the relationship between \
                   sleep quality and academic performance among university students.\n\n\
                   A cross-sectional survey was conducted with 200 participants. \
                   Sleep quality was assessed using the Pittsburgh Sleep Quality Index (PSQI). \
                   Academic performance was measured by self-reported GPA.\n\n\
                   Results showed a significant positive correlation between sleep quality \
                   and academic performance (r = 0.45, p < 0.001)."
            .to_string(),
        sections: vec![
            Section {
                heading: "Abstract".to_string(),
                body: "This study examined the relationship between sleep quality and academic performance among university students. A cross-sectional survey was conducted with 200 participants. Results showed a significant positive correlation (r = 0.45, p < 0.001).".to_string(),
            },
            Section {
                heading: "Introduction".to_string(),
                body: "Sleep quality has been increasingly recognized as an important factor affecting student well-being and academic outcomes.".to_string(),
            },
            Section {
                heading: "Methods".to_string(),
                body: "A cross-sectional survey was conducted with 200 participants. Sleep quality was assessed using the Pittsburgh Sleep Quality Index (PSQI).".to_string(),
            },
            Section {
                heading: "Results".to_string(),
                body: "Results showed a significant positive correlation between sleep quality and academic performance (r = 0.45, p < 0.001).".to_string(),
            },
        ],
        paragraph_count: 4,
        char_count: 420,
    })
}

#[tauri::command]
pub fn get_dummy_summary() -> Result<SummaryResult, String> {
    Ok(SummaryResult {
        research_topic: "Relationship between sleep quality and academic performance in university students".to_string(),
        objective: "To examine the association between sleep quality and GPA among undergraduate students".to_string(),
        sample_summary: "200 university students, ages 18-24, cross-sectional survey".to_string(),
        design: "Cross-sectional survey study".to_string(),
        methods_summary: "Self-administered questionnaire including PSQI and self-reported GPA".to_string(),
        measures: "Pittsburgh Sleep Quality Index (PSQI), Self-reported GPA".to_string(),
        statistics: "Pearson correlation, multiple regression analysis".to_string(),
        findings: "Significant positive correlation between sleep quality and GPA (r=0.45, p<0.001). Good sleepers had significantly higher GPAs than poor sleepers.".to_string(),
        claimed_contributions: "First study to examine this relationship in the local population with validated instruments".to_string(),
        keywords_for_search: vec![
            "sleep quality".to_string(),
            "academic performance".to_string(),
            "university students".to_string(),
            "GPA".to_string(),
            "PSQI".to_string(),
        ],
    })
}

#[tauri::command]
pub fn get_dummy_journals() -> Result<Vec<JournalCandidate>, String> {
    Ok(vec![
        JournalCandidate {
            journal_name: "Journal of Sleep Research".to_string(),
            publisher: "Wiley".to_string(),
            impact_factor: "3.9".to_string(),
            submission_fee: "$3,150".to_string(),
            match_rate: "92%".to_string(),
            reason: "Strong fit: focuses on sleep research with clinical and population-based studies. The PSQI methodology aligns well with this journal's scope.".to_string(),
        },
        JournalCandidate {
            journal_name: "Sleep Medicine".to_string(),
            publisher: "Elsevier".to_string(),
            impact_factor: "3.8".to_string(),
            submission_fee: "$3,200".to_string(),
            match_rate: "88%".to_string(),
            reason: "Good fit: publishes sleep-related research with clinical implications. The academic performance angle adds novelty.".to_string(),
        },
        JournalCandidate {
            journal_name: "Journal of American College Health".to_string(),
            publisher: "Taylor & Francis".to_string(),
            impact_factor: "2.4".to_string(),
            submission_fee: "$2,800".to_string(),
            match_rate: "85%".to_string(),
            reason: "Excellent fit for the university student population. Regularly publishes studies on health behaviors and academic outcomes.".to_string(),
        },
        JournalCandidate {
            journal_name: "Chronobiology International".to_string(),
            publisher: "Taylor & Francis".to_string(),
            impact_factor: "3.0".to_string(),
            submission_fee: "$2,900".to_string(),
            match_rate: "75%".to_string(),
            reason: "Moderate fit: covers circadian rhythms and sleep. The academic performance angle is relevant but not central.".to_string(),
        },
        JournalCandidate {
            journal_name: "BMC Public Health".to_string(),
            publisher: "Springer Nature".to_string(),
            impact_factor: "3.5".to_string(),
            submission_fee: "$2,790".to_string(),
            match_rate: "72%".to_string(),
            reason: "Broad public health scope. The cross-sectional design and student population are suitable. Impact factor is competitive for this study type.".to_string(),
        },
    ])
}
