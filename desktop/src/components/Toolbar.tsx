import { Analysis } from "../types/analysis";
import { formatNumber, formatSize } from "../utils/formatter";

interface ToolbarProps {
    drives: string[];
    selectedDrive: string;

    loading: boolean;

    analysis: Analysis | null;

    onDriveChange: (drive: string) => void;
    onScan: () => void;
}

export default function Toolbar({
    drives,
    selectedDrive,
    loading,
    analysis,
    onDriveChange,
    onScan,
}: ToolbarProps) {
    return (
        <header className="toolbar">
            <div className="toolbar-top">
                <h1>TreeFellas</h1>

                <div className="overview">
                    {analysis ? (
                        <>
                            <span>Size {formatSize(analysis.total_size)}</span>

                            <span>Files {formatNumber(analysis.total_files)}</span>

                            <span>
                                Directories{" "}
                                {formatNumber(analysis.total_directories)}
                            </span>
                        </>
                    ) : (
                        <>
                            <span>Size --</span>
                            <span>Files --</span>
                            <span>Directories --</span>
                        </>
                    )}
                </div>
            </div>

            <div className="toolbar-bottom">
                <div className="toolbar-actions">
                    <select
                        value={selectedDrive}
                        onChange={(e) =>
                            onDriveChange(e.target.value)
                        }
                    >
                        {drives.map((drive) => (
                            <option
                                key={drive}
                                value={drive}
                            >
                                {drive}
                            </option>
                        ))}
                    </select>

                    <button
                        disabled={loading}
                        onClick={onScan}
                    >
                        {loading ? "Scanning..." : "Scan"}
                    </button>
                </div>

                <input
                    type="text"
                    placeholder="Search..."
                />
            </div>
        </header>
    );
}