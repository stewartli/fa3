use std::process::Command;

const DUCKDB_CLI: &str = "/home/stli/duckdb";

#[derive(Debug)]
pub enum ReconError {
    // duckdb cli fails to launch (not on PATH, etc)
    Launch(std::io::Error),
    // duckdb cli fails to exec
    Run(String),
    // serde_json fails to parse duckdb json output
    Parse(serde_json::Error),
    // duckdb tbl column total
    Other(String),
}

impl std::error::Error for ReconError {}

impl std::fmt::Display for ReconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Launch(x) => write!(f, "launch duckdb: {x}"),
            Self::Run(x) => write!(f, "exec duckdb: {x}"),
            Self::Parse(x) => write!(f, "parse duckdb output: {x}"),
            Self::Other(x) => write!(f, "column total: {x}"),
        }
    }
}

impl From<std::io::Error> for ReconError {
    fn from(value: std::io::Error) -> Self {
        Self::Launch(value)
    }
}

impl From<serde_json::Error> for ReconError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value)
    }
}

fn run_sql(db_path: &str, sql: &str) -> Result<String, ReconError> {
    let res = Command::new(DUCKDB_CLI)
        .arg(db_path)
        .arg("-json")
        .arg("-c")
        .arg(sql)
        .output()?;

    if !res.status.success() {
        return Err(ReconError::Run(
            String::from_utf8_lossy(&res.stderr).into_owned(),
        ));
    }

    Ok(String::from_utf8_lossy(&res.stdout).into_owned())
}

fn sanitized_table_name(account: &str) -> String {
    let lower = account.to_lowercase();
    let sanitized: String = lower
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    format!("{sanitized}_tbl")
}

fn sql_in_list(names: &[String]) -> String {
    if names.is_empty() {
        return "('__none__')".to_string();
    }
    // clean up column name in sql
    let quoted = names
        .iter()
        .map(|n| format!("'{}'", n.replace('\'', "''")))
        .collect::<Vec<String>>();

    format!("({})", quoted.join(", "))
}

pub fn reconcile(
    db_path: &str,
    file_path: &str,
    account: &str,
    leaf_names: &[String],
) -> Result<f64, ReconError> {
    // load data to table
    let tbl = sanitized_table_name(account);
    let in_list = sql_in_list(leaf_names);

    let read_fn = if file_path.to_lowercase().ends_with(".xlsx") {
        "read_xlsx"
    } else {
        "read_csv_auto"
    };

    let sql_data_tbl =
        format!("CREATE OR REPLACE TABLE {tbl} AS SELECT * FROM {read_fn}('{file_path}');");

    run_sql(db_path, &sql_data_tbl)?;

    // compute sub-ledger amount
    let sql_total_amount = format!(
        "SELECT SUM(amount) AS total FROM {tbl}
         WHERE subaccount IN {in_list} OR account IN {in_list};"
    );

    parse_col_total(&run_sql(db_path, &sql_total_amount)?)
}

pub fn record_status(
    db_path: &str,
    account: &str,
    coa_amount: f64,
    gl_amount: f64,
    passed: bool,
) -> Result<(), ReconError> {
    // record audit trail
    let account_name_clean = account.replace('\'', "''");
    let status = if passed { "passed" } else { "failed" };

    let sql_update_status = format!(
        "CREATE TABLE IF NOT EXISTS recon_status (
            account_name TEXT PRIMARY KEY,
            status TEXT,
            coa_amount DOUBLE,
            gl_amount DOUBLE,
            checked_at TIMESTAMP
        );
        DELETE FROM recon_status WHERE account_name = '{account_name_clean}';
        INSERT INTO recon_status VALUES ('{account_name_clean}', '{status}', {coa_amount}, {gl_amount}, now());"
    );

    run_sql(db_path, &sql_update_status)?;
    Ok(())
}

fn parse_col_total(json: &str) -> Result<f64, ReconError> {
    let trimmed = json.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(0.0);
    }

    let parsed: serde_json::Value = serde_json::from_str(trimmed)?;
    let total_field = parsed
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("total"))
        .ok_or_else(|| ReconError::Other(format!("no 'total' field in: {trimmed}")))?;

    match total_field {
        x if x.is_null() => Ok(0.0),
        x => x
            .as_f64()
            .ok_or_else(|| ReconError::Other(format!("'total' was not a number in: {trimmed}"))),
    }
}

pub fn load_all_status(db_path: &str) -> Result<Vec<(String, String)>, ReconError> {
    let sql_select_status = "SELECT account_name, status FROM recon_status;";
    let json_output = match run_sql(db_path, sql_select_status) {
        Ok(j) => j,
        Err(ReconError::Run(_)) => return Ok(vec![]),
        Err(e) => return Err(e),
    };

    let trimmed = json_output.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(vec![]);
    }

    let parsed: serde_json::Value = serde_json::from_str(trimmed)?;
    let rows = parsed
        .as_array()
        .ok_or_else(|| ReconError::Other(format!("expected json array: {trimmed}")))?;

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
