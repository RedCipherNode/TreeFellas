import { Analysis } from "../types/analysis";
import { formatNumber, formatSize } from "../utils/formatter";

interface ToolbarProps {
  location: string;
  loading: boolean;
  analysis: Analysis | null;

  onLocationChange: (location: string) => void;
  onBrowse: () => void;
  onScan: () => void;
}

export default function Toolbar({ location, loading, analysis, onLocationChange, onBrowse, onScan }: ToolbarProps) {
  return (
    <header className="toolbar">
      <div className="toolbar-location">
        {" "}
        <div className="location-actions">
          <input className="location-bar" value={location} onChange={(e) => onLocationChange(e.target.value)} />

          <button onClick={onBrowse}>Browse</button>

          <button disabled={loading} onClick={onScan}>
            {loading ? "Scanning..." : "Scan"}
          </button>
        </div>
        <div className="scan-progress">{loading && <div className="scan-progress-fill" />}</div>
      </div>

      <div className="toolbar-overview">
        <div>
          <span>Size </span>

          <strong>{analysis ? formatSize(analysis.total_size) : "--"}</strong>
        </div>

        <div>
          <span>Files </span>

          <strong>{analysis ? formatNumber(analysis.total_files) : "--"}</strong>
        </div>

        <div>
          <span>Dirs </span>

          <strong>{analysis ? formatNumber(analysis.total_directories) : "--"}</strong>
        </div>
      </div>

      <div className="toolbar-extra"></div>
    </header>
  );
}
