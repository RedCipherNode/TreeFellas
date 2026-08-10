# NTFS vs STDFS Benchmark

## C:\

| Scanner | Files | Directories | Total Size | Scan Time |
|---|---:|---:|---:|---:|
| STDFS | 1.14M | 288k | 201.76 GB | 306.58 s |
| NTFS | 1.95M | 443k | 297.25 GB | 20.77 s |
| **Diff (NTFS - STDFS)** | **+809k** | **+156k** | **+95.49 GB** | **-285.81 s** |

## D:\

| Scanner | Files | Directories | Total Size | Scan Time |
|---|---:|---:|---:|---:|
| STDFS | 656k | 118k | 511.26 GB | 116.30 s |
| NTFS | 656k | 118k | 502.35 GB | 6.26 s |
| **Diff (NTFS - STDFS)** | **+35** | **+12** | **-8.91 GB** | **-110.04 s** |