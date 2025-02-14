# IceBucket - Folder Synchronization Tool

**IceBucket** is a simple background service that synchronizes local folders with AWS S3.  
Currently, only **uploading to S3** is implemented, but download and bi-directional sync will be added soon.

## 🚀 Features
- Runs as a **Windows tray application** with a minimal UI.
- **Uploads files to AWS S3** automatically.
- Supports **custom sync intervals** and multiple directories.
- Lightweight, with **no unnecessary dependencies**.

## Plan

- Synchronize files with any external service, including S3, Azure, and even just an FTP server.
- Provide an interface to define which folders to synchronize.
- Provide an interface showing the most recent actions.

---

## 📥 Installation

### **1. Download and Install**
```sh
cargo install icebucket
```

### **2. Run the App**
```sh
icebucket
```
It will start in the system tray.

### **3. (Optional) Add to Startup**
To make IceBucket run at startup:
```sh
icebucket --install
```
To remove from startup:
```sh
icebucket --uninstall
```

---

## ⚙️ Configuration

### **Config File (`settings.json`)**
By default, the app creates a `settings.json` file in the same directory if it doesn't exist.

```json
{
  "service": "s3",
  "access_key": "YOUR_ACCESS_KEY",
  "secret_key": "YOUR_SECRET_KEY",
  "region": "us-east-1",
  "bucket": "your-bucket-name",
  "sync_type": "upload-only",
  "conflicts": "keep-local",
  "directories_to_scan": ["C:/Users/Me/Documents", "D:/Backup"],
  "seconds_between_scans": 60
}
```

### **Configuration Options**
| Key                 | Description |
|---------------------|-------------|
| `service`          | Must be `"s3"` for AWS S3 support. |
| `access_key`       | AWS Access Key ID. |
| `secret_key`       | AWS Secret Access Key. |
| `region`          | AWS region (e.g., `"us-east-1"`). |
| `bucket`          | S3 bucket name where files are uploaded. |
| `sync_type`       | `"upload-only"`, `"download-only"`, or `"upload-and-download"`. |
| `conflicts`       | `"keep-local"` (keep local version) or `"use-remote"` (overwrite with remote). |
| `directories_to_scan` | List of local directories to sync. |
| `seconds_between_scans` | How often (in seconds) to sync changes. |

---

## 📌 Tray Menu Options
Right-click the **IceBucket** tray icon to:
- **Help** - Opens documentation.
- **Exit** - Stops the app.

---

## 🔧 Future Features
- **Download support** from S3.
- **Two-way sync** (upload + download).
- **Custom conflict resolution** settings.
- **Support for Azure Blob Storage & FTP**.

---

## 🛠️ Troubleshooting

### **App Doesn’t Start**
- Check `settings.json` for missing or incorrect fields.
- Run `icebucket --install` if it should start on boot.
- Check logs for errors.

### **Sync Isn’t Working**
- Ensure AWS credentials are correct.
- Verify the S3 bucket exists and has correct permissions.
- Run manually to check for errors.

---

## 📜 License
MIT License. Use freely.
