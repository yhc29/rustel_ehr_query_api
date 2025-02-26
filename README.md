# rustel_ehr_query_api
A RESTful API built with Rust and Rocket that provides access to MIMIC3 database using Temporal Ensemble Logic (TEL).
This API can be used with any Electronic Health Record (EHR) dataset by:
1. Converting your data to the TEL schema format
2. Importing it into MongoDB collections:
   - `cde`: Common Data Elements
   - `tcde`: Temporal Common Data Elements
   - `events`: Patient Events
   - `cde_records`: Patient records stored using cde
   - `event_records`: Patient records stored using events
   - `eii`: Event Inverted index for cohort query
   - `fc`: TEL temporal cohort query structure using Fractional Cascading

## Installation

1. Clone the repository:

2. Generate SSL certificates:
```bash
mkdir -p certs
cd certs
openssl req -x509 -newkey rsa:4096 -nodes -keyout private.key -out cert.pem -days 365 -subj "/CN=localhost"
```

3. Create configuration files:

Create `.env` file:
```env
MONGODB_URI=mongodb://YOUR_MONGO_URI
TEL_DB_NAME=YOUR_TEL_DB_NAME
```

Create `Rocket.toml`:
```toml
[default]
address = "0.0.0.0"
port = 8000
tls = { certs = "certs/cert.pem", key = "certs/private.key" }
```

4. Build and run:
```bash
cargo build --release
cargo run
```

5. Open API Documentation

Get host IP
```bash
ifconfig | grep "inet " | grep -v 127.0.0.1
```
Go to page: https://YOUR_IP_ADDRESS:8000