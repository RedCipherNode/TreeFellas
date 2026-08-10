export interface EntrySummary {
    path: string;
    size: number;
}

export interface ExtensionStatistic {
    extension: string;
    count: number;
    size: number;
}

export interface Analysis {
    total_size: number;
    total_files: number;
    total_directories: number;

    largest_files: EntrySummary[];
    largest_directories: EntrySummary[];

    extension_statistics: ExtensionStatistic[];

    empty_files: EntrySummary[];
    empty_directories: EntrySummary[];
}

export interface Entry {
    name: string;
    path: string;

    is_directory: boolean;

    size: number;

    file_count: number;
    directory_count: number;

    children: Entry[];
}

export interface ScanResult {
    tree: Entry;
    analysis: Analysis;
}