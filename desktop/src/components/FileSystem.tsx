import { useState } from "react";

import { Entry } from "../types/analysis";
import TreeView from "./TreeView";

interface FileSystemProps {
    tree: Entry | null;
}

export default function FileSystem({
    tree,
}: FileSystemProps) {
    const [sort, setSort] = useState("size-desc");

    return (
        <section className="panel filesystem">
            <div className="panel-header">
                <h2>File System</h2>

                <select
                    value={sort}
                    onChange={(e) => setSort(e.target.value)}
                >
                    <option value="size-desc">
                        Size (Largest)
                    </option>

                    <option value="size-asc">
                        Size (Smallest)
                    </option>

                    <option value="name-asc">
                        Name (A-Z)
                    </option>

                    <option value="name-desc">
                        Name (Z-A)
                    </option>
                </select>
            </div>

            <div className="panel-body">
                {tree ? (
                    <table className="tree-table">
                        <thead>
                            <tr>
                                <th>Name</th>
                                <th>Size</th>
                            </tr>
                        </thead>

                        <tbody>
                            <TreeView entry={tree} />
                        </tbody>
                    </table>
                ) : (
                    <div className="empty-state">
                        Scan a drive to display the file system.
                    </div>
                )}
            </div>
        </section>
    );
}