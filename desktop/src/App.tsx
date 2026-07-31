import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import Toolbar from "./components/Toolbar";
import FileSystem from "./components/FileSystem";
import Treemap from "./components/Treemap";

import { ScanResult } from "./types/analysis";

import "./App.css";

export default function App() {
  const [location, setLocation] = useState("");

  const [scanResult, setScanResult] = useState<ScanResult | null>(null);

  const [loading, setLoading] = useState(false);

  useEffect(() => {
    async function loadDefaultLocation() {
      const drives = await invoke<string[]>("get_drives");

      if (drives.length > 0) {
        setLocation(drives[0]);
      }
    }

    loadDefaultLocation();
  }, []);

  async function browse() {
    // TODO:
    // Open folder picker
  }

  async function scan() {
    if (!location.trim()) {
      return;
    }

    setLoading(true);

    try {
      const result = await invoke<ScanResult>("scan", {
        path: location,
      });

      setScanResult(result);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="app">
      <Toolbar location={location} loading={loading} analysis={scanResult?.analysis ?? null} onLocationChange={setLocation} onBrowse={browse} onScan={scan} />

      <FileSystem location={location} tree={scanResult?.tree ?? null} />

      <Treemap analysis={scanResult?.analysis ?? null} />
    </div>
  );
}
