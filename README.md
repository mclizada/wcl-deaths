# wcl-deaths

Analyzes WarcraftLogs reports to surface bad deaths on a specific encounter.

## Setup

Create a `.env` file with your WarcraftLogs API credentials:

```
WCL_CLIENT_ID=your_client_id
WCL_CLIENT_SECRET=your_client_secret
```

Credentials can be obtained from the [WarcraftLogs API client manager](https://www.warcraftlogs.com/api/clients/).

## Usage

Single report:
```
cargo run -- -r AbVphwHqgLJ7ZQ3Y
```

Multiple reports aggregated together:
```
cargo run -- -r AbVphwHqgLJ7ZQ3Y -r GbkAZP4Hwvn68yfL
```

Different encounter:
```
cargo run -- -r AbVphwHqgLJ7ZQ3Y --encounter 3135
```
