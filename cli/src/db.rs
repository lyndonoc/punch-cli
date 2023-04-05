use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

pub fn create_connection() -> Result<SqliteConnection, std::io::Error> {
    Ok(SqliteConnection::establish("./cli/punchcard.db")
        .expect("failed to establish database connection"))
}
