use std::process::Command;

#[derive(Debug)]
pub enum ReconError {
    // duckdb cli fails to launch (not on PATH, etc)
    Spawn(std::io::Error),
    // duckdb cli fails to exec
    Cli(String),
    // serde_json fails to parse duckdb json output
    Parse(String),
}

impl std::error::Error for ReconError {}

impl std::fmt::Display for ReconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconError::Spawn(x) => write!(f, "launch duckdb: {x}"),
            ReconError::Cli(x) => write!(f, "exec duckdb: {x}"),
            ReconError::Parse(x) => write!(f, "parse duckdb output: {x}"),
        }
    }
}

fn run_sql(db_path: &str, sql: &str) -> Result<String, ReconError> {
    let output = Command::new("/home/stli/duckdb")
        .arg(db_path)
        .arg("-json")
        .arg("-c")
        .arg(sql)
        .output()
        .map_err(ReconError::Spawn)?;

    if !output.status.success() {
        return Err(ReconError::Cli(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn sql_in_list(names: &[String]) -> String {
    if names.is_empty() {
        return "('__none__')".to_string();
    }
    let quoted: Vec<String> = names
        .iter()
        .map(|n| format!("'{}'", n.replace('\'', "''")))
        .collect();
    format!("({})", quoted.join(", "))
}

pub fn reconcile(
    db_path: &str,
    file_path: &str,
    account: &str,
    leaf_names: &[String],
    coa_amount: f64,
    passed: bool,
) -> Result<f64, ReconError> {
    // load data to table
    let read_fn = if file_path.to_lowercase().ends_with(".xlsx") {
        "read_xlsx"
    } else {
        "read_csv_auto"
    };

    let account_escaped = account.replace('\'', "''");
    let in_list = sql_in_list(leaf_names);
    let tbl = {
        let lower = account.to_lowercase();
        let sanitized: String = lower
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        format!("{sanitized}_tbl")
    };

    let stage_sql =
        format!("CREATE OR REPLACE TABLE {tbl} AS SELECT * FROM {read_fn}('{file_path}');");

    run_sql(db_path, &stage_sql)?;

    // compute sub-ledger amount
    let sum_sql = format!(
        "SELECT SUM(amount) AS total FROM {tbl}
         WHERE subaccount IN {in_list} OR account IN {in_list};"
    );
    let sum_json = run_sql(db_path, &sum_sql)?;
    let gl_amount = parse_single_total(&sum_json)?;

    // record audit trail
    let status = if passed { "passed" } else { "failed" };
    let upsert_sql = format!(
        "CREATE TABLE IF NOT EXISTS recon_status (
            account_name TEXT PRIMARY KEY,
            status TEXT,
            coa_amount DOUBLE,
            gl_amount DOUBLE,
            checked_at TIMESTAMP
        );
        DELETE FROM recon_status WHERE account_name = '{account_escaped}';
        INSERT INTO recon_status VALUES ('{account_escaped}', '{status}', {coa_amount}, {gl_amount}, now());"
    );

    run_sql(db_path, &upsert_sql)?;

    Ok(gl_amount)
}

fn parse_single_total(json: &str) -> Result<f64, ReconError> {
    let trimmed = json.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(0.0);
    }

    let parsed: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| ReconError::Parse(format!("{e}: {trimmed}")))?;

    let total_field = parsed
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("total"))
        .ok_or_else(|| ReconError::Parse(format!("no 'total' field in: {trimmed}")))?;

    if total_field.is_null() {
        return Ok(0.0);
    }

    total_field
        .as_f64()
        .ok_or_else(|| ReconError::Parse(format!("'total' was not a number in: {trimmed}")))
}

pub fn load_all_status(db_path: &str) -> Result<Vec<(String, String)>, ReconError> {
    let sql = "SELECT account_name, status FROM recon_status;";
    let json = match run_sql(db_path, sql) {
        Ok(j) => j,
        Err(ReconError::Cli(_)) => return Ok(vec![]),
        Err(e) => return Err(e),
    };

    let trimmed = json.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(vec![]);
    }

    let parsed: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| ReconError::Parse(format!("{e}: {trimmed}")))?;

    let rows = parsed
        .as_array()
        .ok_or_else(|| ReconError::Parse(format!("expected array: {trimmed}")))?;

    let mut out = vec![];
    for row in rows {
        let name = row
            .get("account_name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let status = row
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        out.push((name, status));
    }

    Ok(out)
}
