interface ProgressBarProps {
  currentStep: number;
  onStepClick: (step: number) => void;
}

const steps = [
  { label: "テキスト抽出" },
  { label: "論文要約" },
  { label: "立ち位置調査" },
  { label: "ジャーナル調査" },
  { label: "結果出力" },
];

function ProgressBar({ currentStep, onStepClick }: ProgressBarProps) {
  return (
    <div className="progress-bar">
      {steps.map((step, i) => {
        const status = i < currentStep ? "done" : i === currentStep ? "current" : "pending";
        return (
          <div key={i} className="progress-step-wrapper" onClick={() => onStepClick(i)}>
            <div className={`progress-step ${status}`}>
              {status === "done" ? "✓" : i + 1}
            </div>
            <span className={`progress-label ${status}`}>{step.label}</span>
            {i < steps.length - 1 && <div className={`progress-connector ${i < currentStep ? "done" : ""}`} />}
          </div>
        );
      })}
    </div>
  );
}

export default ProgressBar;
