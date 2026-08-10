import { Analysis } from "../types/analysis";
import { formatNumber, formatSize } from "../utils/formatter";

interface OverviewProps {
    analysis: Analysis | null;
}

export default function Overview({ analysis }: OverviewProps) {
    if (!analysis) {
        return null;
    }

    return (
        <div style={{ marginTop: 20 }}>
            <p>
                Size {formatSize(analysis.total_size)}
                {" • "}
                Files {formatNumber(analysis.total_files)}
                {" • "}
                Directories {formatNumber(analysis.total_directories)}
            </p>
        </div>
    );
}