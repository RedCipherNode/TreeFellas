export function formatSize(bytes: number): string {
    const units = ["B", "KB", "MB", "GB", "TB"];

    let size = bytes;
    let unit = 0;

    while (size >= 1024 && unit < units.length - 1) {
        size /= 1024;
        unit++;
    }

    return `${size.toFixed(2)} ${units[unit]}`;
}

export function formatNumber(value: number): string {
    return value.toLocaleString();
}