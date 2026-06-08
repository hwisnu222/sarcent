# Sarcent

**Sarcent** is a tool that allows multiple devices to form a cluster for distributed file sharing.

If one device is running low on storage, the master server will automatically move files to another device in the cluster that still has available space.

Sarcent operates in two modes: **Master** and **Worker**.

- **Master**: Manages and distributes files. Files from the source folder are distributed randomly to all connected workers.
- **Worker**: Receives and stores files sent by the master.

## Installation

Run the following command:

```bash
curl -fsSL https://raw.githubusercontent.com/hwisnu222/sarcent/main/install.sh | sudo bash
```

### Running Worker

On each worker server:

```bash
sarcent worker --target storage/
```

### Running Master

1. First, add the worker nodes to the master:

```bash
sarcent node add --ips=10.1.1.234:50051,200.23.34.123:50051
```

2. Then start the master:

```bash
sarcent master --source directory/
```

## Features

- List of connected devices
- Automatic file distribution when storage is nearly full
- Cluster management (master & workers)
