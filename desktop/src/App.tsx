import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import Toolbar from "./components/Toolbar";
import FileSystem from "./components/FileSystem";
import Treemap from "./components/Treemap";

import { ScanResult } from "./types/analysis";

import "./App.css";

function App() {
    const [drives, setDrives] = useState<string[]>([]);
    const [selectedDrive, setSelectedDrive] = useState("");

    const [scanResult, setScanResult] =
        useState<ScanResult | null>(null);

    const [loading, setLoading] = useState(false);

    useEffect(() => {
        invoke<string[]>("get_drives").then((result) => {
            setDrives(result);

            if (result.length > 0) {
                setSelectedDrive(result[0]);
            }
        });
    }, []);

    async function scan() {
        setLoading(true);

        try {
            const result = await invoke<ScanResult>("scan", {
                path: selectedDrive,
            });

            setScanResult(result);
        } finally {
            setLoading(false);
        }
    }

    return (
        <div className="app">
            <Toolbar
                drives={drives}
                selectedDrive={selectedDrive}
                loading={loading}
                analysis={scanResult?.analysis ?? null}
                onDriveChange={setSelectedDrive}
                onScan={scan}
            />

            <main className="main">
                <FileSystem
                    tree={scanResult?.tree ?? null}
                />

                <Treemap />
            </main>
        </div>
    );
}

export default App;