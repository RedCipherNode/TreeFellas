import { useState } from "react";

import { Entry } from "../types/analysis";
import { formatSize } from "../utils/formatter";

interface TreeViewProps {
    entry: Entry;
    depth?: number;
}

export default function TreeView({
    entry,
    depth = 0,
}: TreeViewProps) {
    const [expanded, setExpanded] = useState(depth < 1);

    const isDirectory = entry.is_directory;

    return (
        <>
            <tr className="tree-row">
                <td>
                    <div
                        className="tree-name"
                        style={{
                            paddingLeft: `${depth * 20}px`,
                        }}
                    >
                        {isDirectory && (
                            <button
                                className="tree-toggle"
                                onClick={() =>
                                    setExpanded(!expanded)
                                }
                            >
                                {expanded ? "▼" : "▶"}
                            </button>
                        )}

                        {!isDirectory && (
                            <span className="tree-toggle-placeholder" />
                        )}

                        <span className="tree-icon">
                            {isDirectory ? "📁" : "📄"}
                        </span>

                        <span>{entry.name}</span>
                    </div>
                </td>

                <td>{formatSize(entry.size)}</td>
            </tr>

            {expanded &&
                isDirectory &&
                entry.children.map((child) => (
                    <TreeView
                        key={child.path}
                        entry={child}
                        depth={depth + 1}
                    />
                ))}
        </>
    );
}