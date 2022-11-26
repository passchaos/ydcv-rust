use anyhow::bail;
use anyhow::Result;
use once_cell::sync::OnceCell;
use redb::{Database, ReadableTable, TableDefinition};
use serde_derive::{Deserialize, Serialize};

const TABLE: TableDefinition<str, str> = TableDefinition::new("ydcv");
fn get_db<'a>() -> Result<&'a Database> {
    static DB: OnceCell<Database> = OnceCell::new();

    DB.get_or_try_init(|| {
        let Some(mut dir) = dirs::home_dir() else {
            bail!("no home dir found");
        };

        dir.push("proliferation/english");

        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        dir.push("ydcv.redb");

        let db = unsafe { Database::create(dir, 1024 * 1024)? };

        Ok(db)
    })
}

fn get_value(key: &str) -> Result<Option<String>> {
    let db = get_db()?;

    let read_txn = db.begin_read()?;
    let table = match read_txn.open_table(TABLE) {
        Ok(t) => t,
        Err(e) => match e {
            redb::Error::TableDoesNotExist(msg) => {
                eprintln!("table: {msg} not exist");
                return Ok(None);
            }
            _ => return Err(e.into()),
        },
    };

    let v = table.get(key)?;

    Ok(v.map(|s| s.to_string()))
}

fn insert_value(key: &str, value: &str) -> Result<()> {
    let write_txn = get_db()?.begin_write()?;
    {
        let mut table = write_txn.open_table(TABLE)?;
        table.insert(key, value)?;
    }
    write_txn.commit()?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Answer {
    pub explain: String,
    pub query_count: u64,
}

pub fn save_query_explain(query: &str, explain: String) -> Result<Answer> {
    let saved_answer = get_value(query)?;

    let query_count = if let Some(answer) = saved_answer {
        let answer: Answer = serde_json::from_str(&answer)?;

        answer.query_count + 1
    } else {
        1
    };

    let answer = Answer {
        explain,
        query_count,
    };

    insert_value(query, &serde_json::to_string(&answer)?)?;

    Ok(answer)
}
